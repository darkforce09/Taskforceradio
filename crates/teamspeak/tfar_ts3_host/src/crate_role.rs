//! The one-line statement of what `tfar_ts3_host` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_ts3_host` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_ts3_host: the boundary to TeamSpeak: the ts3plugin_* export \
     layer over ts3plugin-sys, a safe wrapper around the Ts3Functions \
     table, C-string ownership, and a tracing layer that forwards to \
     TeamSpeak's log.";
