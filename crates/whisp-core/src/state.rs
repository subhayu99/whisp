use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::context::ContextEngine;
use crate::input::InputMonitor;
use crate::llm::LlmBridge;
use crate::privacy::PrivacyGuard;

/// Global application state shared across modules.
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub input_monitor: Arc<InputMonitor>,
    pub privacy_guard: Arc<PrivacyGuard>,
    pub context_engine: Arc<ContextEngine>,
    pub llm_bridge: Arc<LlmBridge>,
    pub paused: Arc<RwLock<bool>>,
}
