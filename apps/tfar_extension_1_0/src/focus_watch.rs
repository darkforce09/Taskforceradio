//! When the 1.0 extension reads the foreground window and when it sends a `FOCUS` frame.

use std::time::{Duration, Instant};

#[cfg(test)]
#[path = "tests/focus_watch.rs"]
mod tests;

/// The shortest time between two reads of the foreground window.
pub const FOCUS_READ_INTERVAL: Duration = Duration::from_millis(250);

/// The focus state the extension last read and last handed to the client.
///
/// [`FocusWatch::poll`] reads the foreground window at most once per [`FOCUS_READ_INTERVAL`] and
/// reports a value to send only when "is Arma in front" changed since the last value it reported
/// (the first read always reports). The client keeps the reported value and sends it again after
/// every new handshake, so a value whose send failed still reaches the next session.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FocusWatch {
    /// When the foreground window was read last.
    last_read: Option<Instant>,
    /// The value reported last.
    last_reported: Option<bool>,
}

impl FocusWatch {
    /// A watch that has read nothing yet.
    pub const fn new() -> Self {
        Self {
            last_read: None,
            last_reported: None,
        }
    }

    /// At `now`, reads the focus with `read_focus` when the read interval has passed, and returns
    /// the value to send when it differs from the value reported last.
    pub fn poll(&mut self, now: Instant, read_focus: impl FnOnce() -> bool) -> Option<bool> {
        if let Some(last_read) = self.last_read
            && now.saturating_duration_since(last_read) < FOCUS_READ_INTERVAL
        {
            return None;
        }
        self.last_read = Some(now);
        let arma_focused = read_focus();
        if self.last_reported == Some(arma_focused) {
            return None;
        }
        self.last_reported = Some(arma_focused);
        Some(arma_focused)
    }
}
