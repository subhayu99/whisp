mod commands;

use whisp_core::config;
use whisp_core::llm::ollama::OllamaBackend;
use whisp_core::llm::LlmBridge;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let _app_handle = app.handle().clone();

            // Load or create config
            let app_config =
                config::load_or_create_default().expect("Failed to load configuration");

            // Initialize LLM backend
            let data_dir = config::data_dir();
            let ollama_backend = OllamaBackend::new(data_dir);
            let llm_bridge = Arc::new(LlmBridge::new(Box::new(ollama_backend)));

            // Store in Tauri state
            app.manage(llm_bridge);
            app.manage(Arc::new(tokio::sync::RwLock::new(app_config)));

            // If first launch, show setup window
            // Otherwise, start the daemon in background
            log::info!("Whisp started");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::toggle_pause,
            commands::get_config,
            commands::setup_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Whisp");
}
