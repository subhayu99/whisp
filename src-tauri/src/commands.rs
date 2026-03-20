use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use whisp_core::config::AppConfig;
use whisp_core::llm::LlmBridge;

#[derive(serde::Serialize)]
pub struct StatusResponse {
    pub running: bool,
    pub paused: bool,
    pub backend: String,
    pub model: String,
}

#[tauri::command]
pub async fn get_status(
    config: State<'_, Arc<RwLock<AppConfig>>>,
) -> Result<StatusResponse, String> {
    let cfg = config.read().await;
    Ok(StatusResponse {
        running: cfg.general.enabled,
        paused: false, // TODO: wire up actual pause state
        backend: cfg.llm.provider.clone(),
        model: cfg.llm.model.clone(),
    })
}

#[tauri::command]
pub async fn toggle_pause() -> Result<bool, String> {
    // TODO: toggle the daemon pause state
    Ok(false)
}

#[tauri::command]
pub async fn get_config(config: State<'_, Arc<RwLock<AppConfig>>>) -> Result<AppConfig, String> {
    let cfg = config.read().await;
    Ok(cfg.clone())
}

#[derive(serde::Deserialize)]
pub struct SetupModelRequest {
    pub model: String,
}

#[tauri::command]
pub async fn setup_model(
    request: SetupModelRequest,
    config: State<'_, Arc<RwLock<AppConfig>>>,
    _llm: State<'_, Arc<LlmBridge>>,
) -> Result<String, String> {
    // Detect if the chosen model supports vision
    let vision_models = [
        "moondream",
        "llava",
        "minicpm-v",
        "llava-llama3",
        "llava-phi3",
        "bakllava",
        "cogvlm",
        "internvl",
    ];
    let is_vision = vision_models
        .iter()
        .any(|v| request.model.to_lowercase().contains(v));

    // Update config with chosen model
    {
        let mut cfg = config.write().await;
        cfg.llm.model = request.model.clone();
        cfg.llm.model_supports_vision = is_vision;

        // Persist to disk
        let path = whisp_core::config::config_path();
        if let Ok(content) = toml::to_string_pretty(&*cfg) {
            let _ = std::fs::write(&path, content);
        }
    }

    let mode = if is_vision {
        "vision (screenshot context enabled)"
    } else {
        "text-only (screenshot context disabled)"
    };
    Ok(format!("Model set to: {} [{}]", request.model, mode))
}
