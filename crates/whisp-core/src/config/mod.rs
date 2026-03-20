mod schema;

pub use schema::AppConfig;

use std::path::PathBuf;

/// Get the config file path for the current platform.
pub fn config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("whisp");
    config_dir.join("config.toml")
}

/// Get the app data directory for the current platform.
pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("whisp")
}

/// Load config from disk, or create default if it doesn't exist.
pub fn load_or_create_default() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let path = config_path();

    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    } else {
        let config = AppConfig::default();
        // Ensure parent dir exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(&path, content)?;
        Ok(config)
    }
}
