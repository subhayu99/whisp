use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub privacy: PrivacyConfig,
    pub llm: LlmConfig,
    pub overlay: OverlayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub enabled: bool,
    pub suggestion_delay_ms: u64,
    pub max_text_buffer_chars: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub blocked_apps: Vec<String>,
    pub pause_on_password_fields: bool,
    pub allow_cloud_llm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub ollama_host: String,
    /// Whether the selected model supports vision (multimodal).
    /// When false, screenshots are skipped and only typed text is sent.
    #[serde(default)]
    pub model_supports_vision: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub theme: String,
    pub accept_key: String,
    pub dismiss_key: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                enabled: true,
                suggestion_delay_ms: 1500,
                max_text_buffer_chars: 200,
            },
            privacy: PrivacyConfig {
                blocked_apps: vec![
                    "1Password".to_string(),
                    "Bitwarden".to_string(),
                    "KeePassXC".to_string(),
                ],
                pause_on_password_fields: true,
                allow_cloud_llm: false,
            },
            llm: LlmConfig {
                provider: "ollama".to_string(),
                model: String::new(), // chosen during first-launch setup
                ollama_host: "http://127.0.0.1:11435".to_string(),
                model_supports_vision: false,
            },
            overlay: OverlayConfig {
                theme: "system".to_string(),
                accept_key: "Tab".to_string(),
                dismiss_key: "Escape".to_string(),
            },
        }
    }
}

impl AppConfig {
    /// Check if this is a fresh install (no model selected yet).
    pub fn is_first_launch(&self) -> bool {
        self.llm.model.is_empty()
    }
}
