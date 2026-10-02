//! The one-line statement of what `tfar_plugin_1_0` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_plugin_1_0` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_plugin_1_0: the TeamSpeak plugin for the 1.0 mod: the \
     ts3plugin_* exports, the game link listener with its async queue, \
     the TeamSpeak event handlers, antennas, SETCFG, the focus-aware \
     channel move and the debug window, wired to the 1.0 crates.";
