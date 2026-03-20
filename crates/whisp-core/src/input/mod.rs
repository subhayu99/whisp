mod classifier;

pub use classifier::KeystrokeClassifier;

use std::sync::Arc;
use tokio::sync::mpsc;

/// Events emitted by the input monitor.
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// User is actively typing meaningful text.
    TypingActive,
    /// User paused typing for the configured delay.
    TypingPaused { buffer: String },
    /// A non-typing key was pressed (modifier, shortcut, navigation).
    NonTyping,
}

/// Monitors global keyboard input and emits classified events.
pub struct InputMonitor {
    event_tx: mpsc::Sender<InputEvent>,
    classifier: Arc<KeystrokeClassifier>,
}

impl InputMonitor {
    pub fn new(event_tx: mpsc::Sender<InputEvent>) -> Self {
        Self {
            event_tx,
            classifier: Arc::new(KeystrokeClassifier::new()),
        }
    }

    /// Start listening for global keyboard events.
    /// Must be called from a thread that can run the platform event loop.
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let tx = self.event_tx.clone();
        let classifier = self.classifier.clone();

        std::thread::spawn(move || {
            rdev::listen(move |event| {
                if let rdev::EventType::KeyPress(key) = event.event_type {
                    let input_event = classifier.classify(key);
                    let _ = tx.blocking_send(input_event);
                }
            })
            .expect("Failed to start keyboard listener");
        });

        Ok(())
    }
}
