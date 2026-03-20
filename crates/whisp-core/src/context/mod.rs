mod debounce;
mod screenshot;

pub use debounce::TypingDebouncer;
pub use screenshot::ScreenCapture;

use crate::privacy::SecureBuffer;

/// Packages text + screenshot + app context for LLM consumption.
pub struct ContextEngine {
    buffer: tokio::sync::Mutex<SecureBuffer>,
    screen_capture: ScreenCapture,
    suggestion_delay_ms: u64,
}

/// The context package sent to the LLM.
pub struct ContextPackage {
    pub text: String,
    pub screenshot: Option<Vec<u8>>,
    pub app_name: String,
    pub window_title: String,
}

impl ContextEngine {
    pub fn new(max_buffer_chars: usize, suggestion_delay_ms: u64) -> Self {
        Self {
            buffer: tokio::sync::Mutex::new(SecureBuffer::new(max_buffer_chars)),
            screen_capture: ScreenCapture::new(),
            suggestion_delay_ms,
        }
    }

    /// Append typed text to the rolling buffer.
    pub async fn append_text(&self, text: &str) {
        let mut buf = self.buffer.lock().await;
        buf.push_str(text);
    }

    /// Build a context package from the current buffer and a screenshot.
    pub async fn build_context(&self) -> Option<ContextPackage> {
        let mut buf = self.buffer.lock().await;
        if buf.is_empty() {
            return None;
        }

        let text = buf.as_str().to_string();
        buf.clear();

        let screenshot = self.screen_capture.capture_active_window().ok();

        Some(ContextPackage {
            text,
            screenshot,
            // TODO: get from platform accessibility API
            app_name: String::new(),
            window_title: String::new(),
        })
    }

    pub fn suggestion_delay_ms(&self) -> u64 {
        self.suggestion_delay_ms
    }
}
