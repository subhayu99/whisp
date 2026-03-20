use zeroize::Zeroize;

/// A text buffer that cryptographically zeroes its memory on drop.
/// Used for all keystroke data to prevent sensitive information
/// from lingering in memory.
#[derive(Clone)]
pub struct SecureBuffer {
    data: Vec<u8>,
    max_len: usize,
}

impl SecureBuffer {
    pub fn new(max_len: usize) -> Self {
        Self {
            data: Vec::with_capacity(max_len),
            max_len,
        }
    }

    pub fn push_str(&mut self, s: &str) {
        self.data.extend_from_slice(s.as_bytes());
        // Trim from the front if over capacity
        if self.data.len() > self.max_len {
            let drain_to = self.data.len() - self.max_len;
            self.data.drain(..drain_to);
        }
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.data).unwrap_or("")
    }

    pub fn clear(&mut self) {
        self.data.zeroize();
        self.data.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}
