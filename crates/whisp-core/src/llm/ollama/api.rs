use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::llm::{CompletionChunk, CompletionRequest, LlmError, ModelInfo, ProgressCallback};

/// Check if Ollama is healthy at the given host.
pub async fn check_health(host: &str) -> Result<String, LlmError> {
    let url = format!("{}/api/version", host);
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| LlmError::NetworkError(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(LlmError::NotRunning);
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| LlmError::NetworkError(e.to_string()))?;

    Ok(body["version"].as_str().unwrap_or("unknown").to_string())
}

/// List locally available models.
pub async fn list_models(host: &str) -> Result<Vec<ModelInfo>, LlmError> {
    let url = format!("{}/api/tags", host);
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| LlmError::NetworkError(e.to_string()))?;

    let body: TagsResponse = resp
        .json()
        .await
        .map_err(|e| LlmError::NetworkError(e.to_string()))?;

    Ok(body
        .models
        .into_iter()
        .map(|m| ModelInfo {
            name: m.name,
            size_bytes: m.size,
            modified_at: m.modified_at,
        })
        .collect())
}

/// Pull a model, reporting progress via callback.
pub async fn pull_model(
    host: &str,
    model: &str,
    progress: Option<ProgressCallback>,
) -> Result<(), LlmError> {
    let url = format!("{}/api/pull", host);
    let client = reqwest::Client::new();

    let resp = client
        .post(&url)
        .json(&PullRequest {
            name: model.to_string(),
            stream: true,
        })
        .send()
        .await
        .map_err(|e| LlmError::DownloadError(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(LlmError::DownloadError(format!(
            "Pull failed with status: {}",
            resp.status()
        )));
    }

    // Stream the pull progress
    use futures::StreamExt;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| LlmError::DownloadError(e.to_string()))?;
        // Each line is a JSON object with status and progress
        for line in String::from_utf8_lossy(&chunk).lines() {
            if line.is_empty() {
                continue;
            }
            if let Ok(status) = serde_json::from_str::<PullStatus>(line) {
                if let Some(ref cb) = progress {
                    let pct = if status.total > 0 {
                        status.completed as f32 / status.total as f32
                    } else {
                        0.0
                    };
                    cb(pct, &status.status);
                }
            }
        }
    }

    Ok(())
}

/// Stream a completion from Ollama.
pub async fn generate_stream(
    host: &str,
    request: CompletionRequest,
) -> Result<Pin<Box<dyn Stream<Item = Result<CompletionChunk, LlmError>> + Send>>, LlmError> {
    let url = format!("{}/api/generate", host);
    let client = reqwest::Client::new();

    let ollama_req = GenerateRequest {
        model: request.model,
        prompt: request.prompt,
        system: request.system,
        stream: true,
        options: Some(GenerateOptions {
            temperature: Some(request.temperature),
            num_predict: Some(request.max_tokens as i32),
            stop: if request.stop_sequences.is_empty() {
                None
            } else {
                Some(request.stop_sequences)
            },
        }),
    };

    let resp = client
        .post(&url)
        .json(&ollama_req)
        .send()
        .await
        .map_err(|e| LlmError::NetworkError(e.to_string()))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(LlmError::InferenceError(format!(
            "Generate failed: {}",
            body
        )));
    }

    use futures::StreamExt;
    let stream = resp.bytes_stream().map(move |chunk| {
        let chunk = chunk.map_err(|e| LlmError::NetworkError(e.to_string()))?;
        let text = String::from_utf8_lossy(&chunk);

        // Parse the last complete JSON line
        let mut last_chunk = CompletionChunk {
            text: String::new(),
            done: false,
        };

        for line in text.lines() {
            if line.is_empty() {
                continue;
            }
            if let Ok(resp) = serde_json::from_str::<GenerateResponse>(line) {
                last_chunk.text.push_str(&resp.response);
                last_chunk.done = resp.done;
            }
        }

        Ok(last_chunk)
    });

    Ok(Box::pin(stream))
}

// --- Ollama API types ---

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<TagModel>,
}

#[derive(Deserialize)]
struct TagModel {
    name: String,
    size: u64,
    modified_at: String,
}

#[derive(Serialize)]
struct PullRequest {
    name: String,
    stream: bool,
}

#[derive(Deserialize)]
struct PullStatus {
    status: String,
    #[serde(default)]
    total: u64,
    #[serde(default)]
    completed: u64,
}

#[derive(Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<GenerateOptions>,
}

#[derive(Serialize)]
struct GenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
    done: bool,
}
