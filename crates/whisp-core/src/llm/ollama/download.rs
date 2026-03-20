use std::path::Path;

use crate::llm::{LlmError, ProgressCallback};

/// Download the Ollama binary for the current platform.
pub async fn download_ollama(
    bin_dir: &Path,
    progress: Option<&ProgressCallback>,
) -> Result<(), LlmError> {
    tokio::fs::create_dir_all(bin_dir)
        .await
        .map_err(|e| LlmError::DownloadError(format!("Failed to create bin dir: {}", e)))?;

    let (url, archive_type) = platform_download_url();

    if let Some(cb) = progress {
        cb(0.05, &format!("Downloading from {}", url));
    }

    // Download the archive
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| LlmError::DownloadError(format!("HTTP request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(LlmError::DownloadError(format!(
            "Download failed with status: {}",
            response.status()
        )));
    }

    let _total_size = response.content_length().unwrap_or(0);
    let bytes = response
        .bytes()
        .await
        .map_err(|e| LlmError::DownloadError(format!("Failed to read response: {}", e)))?;

    if let Some(cb) = progress {
        cb(0.5, "Download complete, extracting...");
    }

    // Extract based on archive type
    match archive_type {
        ArchiveType::TarGz => extract_tar_gz(&bytes, bin_dir)?,
        ArchiveType::TarZst => extract_tar_zst(&bytes, bin_dir)?,
        ArchiveType::Zip => extract_zip(&bytes, bin_dir)?,
    }

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let binary_path = bin_dir.join("ollama");
        if binary_path.exists() {
            std::fs::set_permissions(&binary_path, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| {
                    LlmError::DownloadError(format!("Failed to set permissions: {}", e))
                })?;
        }
    }

    if let Some(cb) = progress {
        cb(1.0, "Ollama installed successfully");
    }

    log::info!("Ollama binary installed to {}", bin_dir.display());
    Ok(())
}

#[allow(dead_code)]
enum ArchiveType {
    TarGz,
    TarZst,
    Zip,
}

fn platform_download_url() -> (&'static str, ArchiveType) {
    #[cfg(target_os = "macos")]
    {
        (
            "https://ollama.com/download/ollama-darwin.tgz",
            ArchiveType::TarGz,
        )
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        (
            "https://ollama.com/download/ollama-linux-amd64.tar.zst",
            ArchiveType::TarZst,
        )
    }

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        (
            "https://ollama.com/download/ollama-linux-arm64.tar.zst",
            ArchiveType::TarZst,
        )
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        (
            "https://ollama.com/download/ollama-windows-amd64.zip",
            ArchiveType::Zip,
        )
    }

    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        (
            "https://ollama.com/download/ollama-windows-arm64.zip",
            ArchiveType::Zip,
        )
    }
}

fn extract_tar_gz(data: &[u8], dest: &Path) -> Result<(), LlmError> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let decoder = GzDecoder::new(data);
    let mut archive = Archive::new(decoder);

    // Extract only the ollama binary
    for entry in archive
        .entries()
        .map_err(|e| LlmError::DownloadError(format!("Failed to read archive: {}", e)))?
    {
        let mut entry =
            entry.map_err(|e| LlmError::DownloadError(format!("Archive entry error: {}", e)))?;
        let path = entry
            .path()
            .map_err(|e| LlmError::DownloadError(format!("Path error: {}", e)))?;

        // Look for the ollama binary in the archive
        if let Some(name) = path.file_name() {
            if name == "ollama" {
                entry
                    .unpack(dest.join("ollama"))
                    .map_err(|e| LlmError::DownloadError(format!("Extraction failed: {}", e)))?;
                return Ok(());
            }
        }
    }

    Err(LlmError::DownloadError(
        "Ollama binary not found in archive".to_string(),
    ))
}

#[cfg(target_os = "linux")]
fn extract_tar_zst(data: &[u8], dest: &Path) -> Result<(), LlmError> {
    use tar::Archive;

    let decoder = zstd::Decoder::new(data)
        .map_err(|e| LlmError::DownloadError(format!("Zstd decode error: {}", e)))?;
    let mut archive = Archive::new(decoder);

    for entry in archive
        .entries()
        .map_err(|e| LlmError::DownloadError(format!("Failed to read archive: {}", e)))?
    {
        let mut entry =
            entry.map_err(|e| LlmError::DownloadError(format!("Archive entry error: {}", e)))?;
        let path = entry
            .path()
            .map_err(|e| LlmError::DownloadError(format!("Path error: {}", e)))?;

        if let Some(name) = path.file_name() {
            if name == "ollama" {
                entry
                    .unpack(dest.join("ollama"))
                    .map_err(|e| LlmError::DownloadError(format!("Extraction failed: {}", e)))?;
                return Ok(());
            }
        }
    }

    Err(LlmError::DownloadError(
        "Ollama binary not found in archive".to_string(),
    ))
}

#[cfg(not(target_os = "linux"))]
fn extract_tar_zst(_data: &[u8], _dest: &Path) -> Result<(), LlmError> {
    Err(LlmError::DownloadError(
        "tar.zst extraction only supported on Linux".to_string(),
    ))
}

#[cfg(target_os = "windows")]
fn extract_zip(data: &[u8], dest: &Path) -> Result<(), LlmError> {
    use std::io::Cursor;

    let reader = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| LlmError::DownloadError(format!("Zip error: {}", e)))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| LlmError::DownloadError(format!("Zip entry error: {}", e)))?;

        if let Some(name) = file.enclosed_name().and_then(|p| p.file_name()) {
            if name == "ollama.exe" {
                let out_path = dest.join("ollama.exe");
                let mut out_file = std::fs::File::create(&out_path)
                    .map_err(|e| LlmError::DownloadError(format!("File create error: {}", e)))?;
                std::io::copy(&mut file, &mut out_file)
                    .map_err(|e| LlmError::DownloadError(format!("Write error: {}", e)))?;
                return Ok(());
            }
        }
    }

    Err(LlmError::DownloadError(
        "ollama.exe not found in archive".to_string(),
    ))
}

#[cfg(not(target_os = "windows"))]
fn extract_zip(_data: &[u8], _dest: &Path) -> Result<(), LlmError> {
    Err(LlmError::DownloadError(
        "Zip extraction only supported on Windows".to_string(),
    ))
}
