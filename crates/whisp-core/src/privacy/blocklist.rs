/// App blocklist — capture is disabled when any of these apps have focus.
pub struct Blocklist {
    apps: Vec<String>,
}

impl Blocklist {
    pub fn new(apps: Vec<String>) -> Self {
        Self { apps }
    }

    pub fn is_blocked(&self, app_name: &str) -> bool {
        let app_lower = app_name.to_lowercase();
        self.apps
            .iter()
            .any(|blocked| app_lower.contains(&blocked.to_lowercase()))
    }

    pub fn add(&mut self, app: String) {
        if !self.is_blocked(&app) {
            self.apps.push(app);
        }
    }

    pub fn remove(&mut self, app: &str) {
        self.apps.retain(|a| a.to_lowercase() != app.to_lowercase());
    }
}
