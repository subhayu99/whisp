use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Debounces typing events — fires after a configurable pause in typing.
pub struct TypingDebouncer {
    delay: Duration,
}

impl TypingDebouncer {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            delay: Duration::from_millis(delay_ms),
        }
    }

    /// Runs the debounce loop. Receives typing events, emits a "paused" signal
    /// after `delay` ms of silence.
    pub async fn run(&self, mut typing_rx: mpsc::Receiver<()>, pause_tx: mpsc::Sender<()>) {
        loop {
            // Wait for first typing event
            if typing_rx.recv().await.is_none() {
                break;
            }

            // Drain any buffered events and wait for silence
            loop {
                tokio::select! {
                    result = typing_rx.recv() => {
                        if result.is_none() {
                            return;
                        }
                        // Got another keystroke, restart the timer
                        continue;
                    }
                    _ = sleep(self.delay) => {
                        // Silence detected — user paused typing
                        let _ = pause_tx.send(()).await;
                        break;
                    }
                }
            }
        }
    }
}
