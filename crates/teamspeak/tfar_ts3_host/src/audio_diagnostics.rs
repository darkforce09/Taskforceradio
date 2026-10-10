//! `AudioDiagnostics`: counters a plugin's audio callbacks bump and its service thread logs:
//! the voice blocks that waited for the plugin's state and the blocks passed through unprocessed.

use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(test)]
#[path = "tests/audio_diagnostics.rs"]
mod tests;

/// Lock-free counters of the audio callbacks' lock waits and skipped blocks.
#[derive(Debug, Default)]
pub struct AudioDiagnostics {
    contended_state_locks: AtomicU64,
    skipped_blocks: AtomicU64,
}

/// The counts taken out of [`AudioDiagnostics`] since the last take.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AudioDiagnosticCounts {
    /// Voice blocks whose callback found a connection's state locked and waited for it.
    pub contended_state_locks: u64,
    /// Blocks passed through because the server map was being changed.
    pub skipped_blocks: u64,
}

impl AudioDiagnostics {
    /// Zeroed counters.
    pub const fn new() -> Self {
        Self {
            contended_state_locks: AtomicU64::new(0),
            skipped_blocks: AtomicU64::new(0),
        }
    }

    /// Counts a voice block that had to wait for a connection's state.
    pub fn record_contended_state_lock(&self) {
        self.contended_state_locks.fetch_add(1, Ordering::Relaxed);
    }

    /// Counts a block passed through because the server map was busy.
    pub fn record_skipped_block(&self) {
        self.skipped_blocks.fetch_add(1, Ordering::Relaxed);
    }

    /// The counts since the last take, resetting them to zero.
    pub fn take(&self) -> AudioDiagnosticCounts {
        AudioDiagnosticCounts {
            contended_state_locks: self.contended_state_locks.swap(0, Ordering::Relaxed),
            skipped_blocks: self.skipped_blocks.swap(0, Ordering::Relaxed),
        }
    }
}

impl AudioDiagnosticCounts {
    /// Whether every count is zero.
    pub const fn is_zero(&self) -> bool {
        self.contended_state_locks == 0 && self.skipped_blocks == 0
    }
}
