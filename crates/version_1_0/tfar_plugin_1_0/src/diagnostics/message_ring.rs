//! [`MessageRing`]: the latest game messages, kept in a ring of fixed capacity for the
//! diagnostics dump.

use std::collections::VecDeque;

#[cfg(test)]
#[path = "tests/message_ring.rs"]
mod tests;

/// The longest message kept, in bytes; longer ones are cut at a character boundary.
pub const MAX_MESSAGE_BYTES: usize = 512;

/// The newest game messages, oldest first, at most `capacity` of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRing {
    capacity: usize,
    messages: VecDeque<String>,
}

impl MessageRing {
    /// An empty ring keeping at most `capacity` messages.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            messages: VecDeque::with_capacity(capacity),
        }
    }

    /// Keeps `message`, dropping the oldest when the ring is full.
    pub fn push(&mut self, message: &str) {
        if self.capacity == 0 {
            return;
        }
        while self.messages.len() >= self.capacity {
            self.messages.pop_front();
        }
        let mut end = message.len().min(MAX_MESSAGE_BYTES);
        while !message.is_char_boundary(end) {
            end = end.saturating_sub(1);
        }
        self.messages
            .push_back(message.get(..end).unwrap_or_default().to_owned());
    }

    /// The kept messages, oldest first.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.messages.iter().map(String::as_str)
    }

    /// How many messages are kept.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Whether no message is kept.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}
