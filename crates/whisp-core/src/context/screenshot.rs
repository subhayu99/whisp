use image::DynamicImage;

/// Captures screenshots of the active window for LLM context.
pub struct ScreenCapture;

impl ScreenCapture {
    pub fn new() -> Self {
        Self
    }

    /// Capture the active window and return as JPEG bytes (resized to 720p).
    pub fn capture_active_window(&self) -> Result<Vec<u8>, ScreenCaptureError> {
        let monitors = xcap::Monitor::all().map_err(|e| {
            ScreenCaptureError::CaptureError(format!("Failed to enumerate monitors: {}", e))
        })?;

        let monitor = monitors
            .into_iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .ok_or(ScreenCaptureError::NoMonitor)?;

        let raw_image = monitor
            .capture_image()
            .map_err(|e| ScreenCaptureError::CaptureError(e.to_string()))?;

        let img = DynamicImage::ImageRgba8(raw_image);
        // Resize to 720p max height while preserving aspect ratio
        let resized = img.resize(1280, 720, image::imageops::FilterType::Triangle);

        let mut jpeg_bytes = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut jpeg_bytes);
        resized
            .write_to(&mut cursor, image::ImageFormat::Jpeg)
            .map_err(|e| ScreenCaptureError::EncodeError(e.to_string()))?;

        Ok(jpeg_bytes)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ScreenCaptureError {
    #[error("No primary monitor found")]
    NoMonitor,
    #[error("Screen capture failed: {0}")]
    CaptureError(String),
    #[error("Image encoding failed: {0}")]
    EncodeError(String),
}
