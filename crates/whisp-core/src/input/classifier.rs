use rdev::Key;
use std::sync::Mutex;
use std::time::Instant;

use super::InputEvent;

/// Classifies keystrokes as meaningful typing vs shortcuts/navigation.
///
/// "Meaningful typing" = 3+ alphanumeric keys within 2 seconds,
/// without modifier keys held.
pub struct KeystrokeClassifier {
    inner: Mutex<ClassifierState>,
}

struct ClassifierState {
    buffer: String,
    key_count: usize,
    window_start: Instant,
}

impl Default for KeystrokeClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl KeystrokeClassifier {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(ClassifierState {
                buffer: String::new(),
                key_count: 0,
                window_start: Instant::now(),
            }),
        }
    }

    pub fn classify(&self, key: Key) -> InputEvent {
        let mut state = self.inner.lock().unwrap();

        // Reset window if more than 2 seconds since first key
        if state.window_start.elapsed().as_secs_f64() > 2.0 {
            state.key_count = 0;
            state.window_start = Instant::now();
        }

        match key_to_char(key) {
            Some(ch) => {
                state.buffer.push(ch);
                // Cap buffer at 200 chars
                if state.buffer.len() > 200 {
                    let drain_to = state.buffer.len() - 200;
                    state.buffer.drain(..drain_to);
                }
                state.key_count += 1;

                if state.key_count >= 3 {
                    InputEvent::TypingActive
                } else {
                    InputEvent::NonTyping
                }
            }
            None => {
                // Modifier, navigation, or special key
                state.key_count = 0;
                InputEvent::NonTyping
            }
        }
    }

    /// Drain the current buffer contents (called on typing pause).
    pub fn drain_buffer(&self) -> String {
        let mut state = self.inner.lock().unwrap();
        let buffer = state.buffer.clone();
        state.buffer.clear();
        state.key_count = 0;
        buffer
    }
}

/// Map rdev keys to characters for buffer building.
fn key_to_char(key: Key) -> Option<char> {
    match key {
        Key::KeyA => Some('a'),
        Key::KeyB => Some('b'),
        Key::KeyC => Some('c'),
        Key::KeyD => Some('d'),
        Key::KeyE => Some('e'),
        Key::KeyF => Some('f'),
        Key::KeyG => Some('g'),
        Key::KeyH => Some('h'),
        Key::KeyI => Some('i'),
        Key::KeyJ => Some('j'),
        Key::KeyK => Some('k'),
        Key::KeyL => Some('l'),
        Key::KeyM => Some('m'),
        Key::KeyN => Some('n'),
        Key::KeyO => Some('o'),
        Key::KeyP => Some('p'),
        Key::KeyQ => Some('q'),
        Key::KeyR => Some('r'),
        Key::KeyS => Some('s'),
        Key::KeyT => Some('t'),
        Key::KeyU => Some('u'),
        Key::KeyV => Some('v'),
        Key::KeyW => Some('w'),
        Key::KeyX => Some('x'),
        Key::KeyY => Some('y'),
        Key::KeyZ => Some('z'),
        Key::Num0 => Some('0'),
        Key::Num1 => Some('1'),
        Key::Num2 => Some('2'),
        Key::Num3 => Some('3'),
        Key::Num4 => Some('4'),
        Key::Num5 => Some('5'),
        Key::Num6 => Some('6'),
        Key::Num7 => Some('7'),
        Key::Num8 => Some('8'),
        Key::Num9 => Some('9'),
        Key::Space => Some(' '),
        Key::Dot => Some('.'),
        Key::Comma => Some(','),
        Key::SemiColon => Some(';'),
        Key::Quote => Some('\''),
        Key::Minus => Some('-'),
        Key::Equal => Some('='),
        Key::Slash => Some('/'),
        _ => None,
    }
}
