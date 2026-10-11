//! The plugin-enabled cache of a client: whether it runs TFAR and plays.

use std::time::{Duration, Instant};

use crate::plugin_commands::metadata_says_playing;

use crate::client_state::client_data::ClientData;

#[cfg(test)]
#[path = "tests/plugin_enabled.rs"]
mod tests;

/// How long a result read from a client's metadata, or set by a plugin command, stands
/// (`isPluginEnabledForUser`, `reference/tfar_1_0/ts/src/plugin.cpp:421-445`).
pub const PLUGIN_ENABLED_CACHE: Duration = Duration::from_secs(10);

impl ClientData {
    /// The cached answer at `now`, or `None` when it is older than [`PLUGIN_ENABLED_CACHE`] (or
    /// was never taken) and the client's metadata must be read again.
    pub fn cached_plugin_enabled(&self, now: Instant) -> Option<bool> {
        let checked_at = self.plugin_enabled_checked_at?;
        (now.saturating_duration_since(checked_at) < PLUGIN_ENABLED_CACHE)
            .then_some(self.plugin_enabled)
    }

    /// Whether the client runs TFAR and plays, at `now`. Within [`PLUGIN_ENABLED_CACHE`] of the
    /// last reading the cached answer stands and `metadata` is not looked at. Otherwise
    /// `metadata` is the client's TeamSpeak metadata just read: a block that says "connected and
    /// playing" is yes, any other non-empty text is no, and both are cached from `now`; absent or
    /// empty metadata is no and is not cached, as the original returns before caching.
    pub fn plugin_enabled_for(&mut self, metadata: Option<&str>, now: Instant) -> bool {
        if let Some(cached) = self.cached_plugin_enabled(now) {
            return cached;
        }
        let Some(metadata) = metadata.filter(|text| !text.is_empty()) else {
            return false;
        };
        self.plugin_enabled = metadata_says_playing(metadata);
        self.plugin_enabled_checked_at = Some(now);
        self.plugin_enabled
    }

    /// Records that the client sent a plugin command at `now`, so it runs TFAR.
    pub const fn mark_plugin_enabled(&mut self, now: Instant) {
        self.plugin_enabled = true;
        self.plugin_enabled_checked_at = Some(now);
    }
}
