pub mod ollama;
pub mod prompt;

use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Pluggable LLM backend trait. Implemented by Ollama (now) and llama.cpp (future).
#[async_trait::async_trait]
pub trait LlmBackend: Send + Sync {
    fn name(&self) -> &str;

    // Lifecycle
    async fn setup(&mut self, progress: Option<ProgressCallback>) -> Result<(), LlmError>;
    async fn start(&mut self) -> Result<(), LlmError>;
    async fn stop(&mut self) -> Result<(), LlmError>;
    async fn health_check(&self) -> Result<BackendStatus, LlmError>;

    // Model management
    async fn list_models(&self) -> Result<Vec<ModelInfo>, LlmError>;
    async fn pull_model(
        &self,
        model: &str,
        progress: Option<ProgressCallback>,
    ) -> Result<(), LlmError>;

    // Inference
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<CompletionChunk, LlmError>> + Send>>, LlmError>;
}

pub type ProgressCallback = Box<dyn Fn(f32, &str) + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub prompt: String,
    pub system: Option<String>,
    pub context_text: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub stop_sequences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChunk {
    pub text: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size_bytes: u64,
    pub modified_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackendStatus {
    NotInstalled,
    Installing { progress: f32 },
    Stopped,
    Starting,
    Ready,
    Error(String),
}

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("Backend not installed")]
    NotInstalled,
    #[error("Backend not running")]
    NotRunning,
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Download failed: {0}")]
    DownloadError(String),
    #[error("Inference error: {0}")]
    InferenceError(String),
    #[error("Process error: {0}")]
    ProcessError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// High-level LLM bridge used by the rest of the app.
pub struct LlmBridge {
    backend: tokio::sync::RwLock<Box<dyn LlmBackend>>,
}

impl LlmBridge {
    pub fn new(backend: Box<dyn LlmBackend>) -> Self {
        Self {
            backend: tokio::sync::RwLock::new(backend),
        }
    }

    pub async fn suggest(&self, text: &str, context: Option<&str>) -> Result<String, LlmError> {
        let backend = self.backend.read().await;
        let request = CompletionRequest {
            model: String::new(), // uses default from config
            prompt: prompt::build_suggestion_prompt(text, context),
            system: Some(prompt::system_prompt().to_string()),
            context_text: context.map(|s| s.to_string()),
            max_tokens: 200,
            temperature: 0.3,
            stop_sequences: vec!["\n\n".to_string()],
        };

        let mut stream = backend.complete(request).await?;
        let mut result = String::new();

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(c) => {
                    result.push_str(&c.text);
                    if c.done {
                        break;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Ok(result)
    }
}
