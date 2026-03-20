use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::sleep;

use crate::llm::{LlmError, ProgressCallback};

const MANAGED_PORT: u16 = 11435;
const HEALTH_TIMEOUT: Duration = Duration::from_secs(30);
const HEALTH_POLL_INTERVAL: Duration = Duration::from_millis(100);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// Manages the Ollama subprocess lifecycle.
pub struct OllamaProcess {
    data_dir: PathBuf,
    child: Option<Child>,
}

impl OllamaProcess {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            child: None,
        }
    }

    pub fn port(&self) -> u16 {
        MANAGED_PORT
    }

    /// Path to the managed Ollama binary.
    fn binary_path(&self) -> PathBuf {
        let bin_name = if cfg!(target_os = "windows") {
            "ollama.exe"
        } else {
            "ollama"
        };
        self.data_dir.join("bin").join(bin_name)
    }

    /// Path to the models directory.
    fn models_dir(&self) -> PathBuf {
        self.data_dir.join("models")
    }

    /// Path to the PID lockfile.
    fn pid_file(&self) -> PathBuf {
        self.data_dir.join("ollama.pid")
    }

    /// Check if the binary has been downloaded.
    pub fn binary_exists(&self) -> bool {
        self.binary_path().exists()
    }

    /// Download the Ollama binary for the current platform.
    pub async fn download_binary(
        &self,
        progress: Option<&ProgressCallback>,
    ) -> Result<(), LlmError> {
        super::download::download_ollama(&self.data_dir.join("bin"), progress).await
    }

    /// Kill any stale Ollama process from a previous run.
    pub async fn cleanup_stale(&self) -> Result<(), LlmError> {
        let pid_file = self.pid_file();
        if !pid_file.exists() {
            return Ok(());
        }

        let pid_str = tokio::fs::read_to_string(&pid_file)
            .await
            .unwrap_or_default();

        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            log::info!("Found stale PID file with PID {}, cleaning up", pid);
            #[cfg(unix)]
            {
                use std::process::Command as StdCommand;
                // Check if process is still alive
                let alive = StdCommand::new("kill")
                    .args(["-0", &pid.to_string()])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);

                if alive {
                    // SIGTERM first
                    let _ = StdCommand::new("kill").args([&pid.to_string()]).status();
                    sleep(Duration::from_secs(2)).await;
                    // Force kill if still alive
                    let _ = StdCommand::new("kill")
                        .args(["-9", &pid.to_string()])
                        .status();
                }
            }
        }

        let _ = tokio::fs::remove_file(&pid_file).await;
        Ok(())
    }

    /// Spawn the Ollama server subprocess.
    pub async fn spawn(&mut self) -> Result<(), LlmError> {
        let binary = self.binary_path();
        if !binary.exists() {
            return Err(LlmError::NotInstalled);
        }

        let models_dir = self.models_dir();
        tokio::fs::create_dir_all(&models_dir)
            .await
            .map_err(|e| LlmError::ProcessError(format!("Failed to create models dir: {}", e)))?;

        let host = format!("127.0.0.1:{}", MANAGED_PORT);

        let child = Command::new(&binary)
            .arg("serve")
            .env("OLLAMA_HOST", &host)
            .env("OLLAMA_MODELS", &models_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| LlmError::ProcessError(format!("Failed to spawn Ollama: {}", e)))?;

        // Write PID lockfile
        if let Some(pid) = child.id() {
            let pid_file = self.pid_file();
            let _ = tokio::fs::write(&pid_file, pid.to_string()).await;
        }

        self.child = Some(child);
        log::info!("Ollama spawned on {}", host);
        Ok(())
    }

    /// Wait for the Ollama server to become healthy.
    pub async fn wait_for_health(&self, host: &str) -> Result<(), LlmError> {
        let start = std::time::Instant::now();

        while start.elapsed() < HEALTH_TIMEOUT {
            if super::api::check_health(host).await.is_ok() {
                log::info!("Ollama is healthy");
                return Ok(());
            }
            sleep(HEALTH_POLL_INTERVAL).await;
        }

        Err(LlmError::ProcessError(
            "Ollama failed to start within 30 seconds".to_string(),
        ))
    }

    /// Gracefully shut down the managed Ollama process.
    pub async fn shutdown(&mut self) -> Result<(), LlmError> {
        if let Some(ref mut child) = self.child {
            #[cfg(unix)]
            {
                if let Some(pid) = child.id() {
                    // Send SIGTERM for graceful shutdown
                    let _ = std::process::Command::new("kill")
                        .args([&pid.to_string()])
                        .status();

                    // Wait up to 5 seconds
                    let start = std::time::Instant::now();
                    while start.elapsed() < SHUTDOWN_TIMEOUT {
                        match child.try_wait() {
                            Ok(Some(_)) => break,
                            Ok(None) => sleep(Duration::from_millis(200)).await,
                            Err(_) => break,
                        }
                    }

                    // Force kill if still alive
                    let _ = child.kill().await;
                }
            }

            #[cfg(not(unix))]
            {
                let _ = child.kill().await;
            }

            let _ = child.wait().await;
        }

        // Clean up PID file
        let _ = tokio::fs::remove_file(self.pid_file()).await;
        self.child = None;

        log::info!("Ollama shut down");
        Ok(())
    }
}

impl Drop for OllamaProcess {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child {
            // Best-effort cleanup in Drop — can't use async here
            #[cfg(unix)]
            {
                if let Some(pid) = child.id() {
                    let _ = std::process::Command::new("kill")
                        .args([&pid.to_string()])
                        .status();
                }
            }
            // start_kill is non-blocking
            let _ = child.start_kill();
        }
    }
}
