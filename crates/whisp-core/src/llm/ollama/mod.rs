pub mod api;
pub mod download;
pub mod process;

use futures::Stream;
use std::pin::Pin;

use super::{
    BackendStatus, CompletionChunk, CompletionRequest, LlmBackend, LlmError, ModelInfo,
    ProgressCallback,
};
use process::OllamaProcess;

/// Ollama-based LLM backend.
///
/// Manages the Ollama subprocess lifecycle and communicates via its REST API.
pub struct OllamaBackend {
    process: OllamaProcess,
    host: String,
    managed: bool,
}

impl OllamaBackend {
    pub fn new(data_dir: std::path::PathBuf) -> Self {
        Self {
            process: OllamaProcess::new(data_dir),
            host: String::new(),
            managed: false,
        }
    }
}

#[async_trait::async_trait]
impl LlmBackend for OllamaBackend {
    fn name(&self) -> &str {
        "Ollama"
    }

    async fn setup(&mut self, progress: Option<ProgressCallback>) -> Result<(), LlmError> {
        // Step 1: Check if user's Ollama is already running
        if api::check_health("http://127.0.0.1:11434").await.is_ok() {
            self.host = "http://127.0.0.1:11434".to_string();
            self.managed = false;
            log::info!("Using existing Ollama instance on port 11434");
            return Ok(());
        }

        // Step 2: Ensure Ollama binary exists, download if not
        if !self.process.binary_exists() {
            if let Some(ref cb) = progress {
                cb(0.0, "Downloading Ollama...");
            }
            self.process.download_binary(progress.as_ref()).await?;
        }

        // Step 3: Clean up stale processes
        self.process.cleanup_stale().await?;

        self.host = format!("http://127.0.0.1:{}", self.process.port());
        self.managed = true;
        Ok(())
    }

    async fn start(&mut self) -> Result<(), LlmError> {
        if !self.managed {
            return Ok(()); // Using external Ollama
        }
        self.process.spawn().await?;
        self.process.wait_for_health(&self.host).await
    }

    async fn stop(&mut self) -> Result<(), LlmError> {
        if self.managed {
            self.process.shutdown().await?;
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<BackendStatus, LlmError> {
        match api::check_health(&self.host).await {
            Ok(_) => Ok(BackendStatus::Ready),
            Err(_) => {
                if self.process.binary_exists() {
                    Ok(BackendStatus::Stopped)
                } else {
                    Ok(BackendStatus::NotInstalled)
                }
            }
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, LlmError> {
        api::list_models(&self.host).await
    }

    async fn pull_model(
        &self,
        model: &str,
        progress: Option<ProgressCallback>,
    ) -> Result<(), LlmError> {
        api::pull_model(&self.host, model, progress).await
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<CompletionChunk, LlmError>> + Send>>, LlmError>
    {
        api::generate_stream(&self.host, request).await
    }
}
