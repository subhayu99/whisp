mod caret;

pub use caret::CaretPosition;

/// Manages the suggestion overlay positioning and visibility.
pub struct OverlayManager {
    visible: bool,
    current_suggestion: Option<String>,
    position: Option<CaretPosition>,
}

impl OverlayManager {
    pub fn new() -> Self {
        Self {
            visible: false,
            current_suggestion: None,
            position: None,
        }
    }

    /// Show a suggestion at the given caret position.
    pub fn show_suggestion(&mut self, suggestion: String, position: Option<CaretPosition>) {
        self.current_suggestion = Some(suggestion);
        self.position = position;
        self.visible = true;
    }

    /// Dismiss the current suggestion.
    pub fn dismiss(&mut self) {
        self.current_suggestion = None;
        self.visible = false;
    }

    /// Accept the current suggestion (returns text to type via input simulation).
    pub fn accept(&mut self) -> Option<String> {
        self.visible = false;
        self.current_suggestion.take()
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn current_suggestion(&self) -> Option<&str> {
        self.current_suggestion.as_deref()
    }

    pub fn position(&self) -> Option<&CaretPosition> {
        self.position.as_ref()
    }
}
