/// Position of the text caret on screen, used to position the overlay.
#[derive(Debug, Clone, Copy)]
pub struct CaretPosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl CaretPosition {
    /// Get the caret position of the focused text element.
    /// Returns None if the caret position cannot be determined.
    pub fn get_focused_caret() -> Option<Self> {
        #[cfg(target_os = "macos")]
        {
            Self::get_macos_caret()
        }

        #[cfg(target_os = "linux")]
        {
            Self::get_linux_caret()
        }

        #[cfg(target_os = "windows")]
        {
            Self::get_windows_caret()
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            None
        }
    }

    #[cfg(target_os = "macos")]
    fn get_macos_caret() -> Option<Self> {
        // TODO: Use AXUIElement API → AXBoundsForRange on focused element
        // This gives us the exact pixel position of the text caret
        None
    }

    #[cfg(target_os = "linux")]
    fn get_linux_caret() -> Option<Self> {
        // TODO: Use AT-SPI2 → GetCharacterExtents
        // Less reliable than macOS, bubble fallback used more often
        None
    }

    #[cfg(target_os = "windows")]
    fn get_windows_caret() -> Option<Self> {
        // TODO: Use UIA → GetBoundingRectangle on caret
        None
    }
}
