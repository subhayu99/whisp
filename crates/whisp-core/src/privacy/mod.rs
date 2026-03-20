mod blocklist;
mod password_detect;
mod secure_buffer;

pub use blocklist::Blocklist;
pub use password_detect::PasswordFieldDetector;
pub use secure_buffer::SecureBuffer;

/// Central privacy guard that decides whether capture should be active.
pub struct PrivacyGuard {
    password_detector: PasswordFieldDetector,
    blocklist: Blocklist,
}

impl PrivacyGuard {
    pub fn new(blocked_apps: Vec<String>) -> Self {
        Self {
            password_detector: PasswordFieldDetector::new(),
            blocklist: Blocklist::new(blocked_apps),
        }
    }

    /// Returns true if capture should be paused (password field or blocked app).
    pub fn should_pause(&self, app_name: &str) -> bool {
        if self.blocklist.is_blocked(app_name) {
            log::info!("Capture paused: app '{}' is blocklisted", app_name);
            return true;
        }

        if self.password_detector.is_password_field_focused() {
            log::info!("Capture paused: password field detected");
            return true;
        }

        false
    }
}
