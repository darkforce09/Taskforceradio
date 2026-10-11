//! `accepts_sender`: which TeamSpeak plugins' commands the 1.0 plugin reads.

#[cfg(test)]
#[path = "tests/sender_filter.rs"]
mod tests;

/// The start of the plugin name of every plugin whose commands the 1.0 plugin reads
/// (`reference/tfar_1_0/ts/src/plugin.cpp:1079`).
pub const PLUGIN_NAME_PREFIX: &str = "TFAR";

/// Whether a plugin command from the plugin named `plugin_name` is read: its name starts with
/// [`PLUGIN_NAME_PREFIX`], letter case included (`strncmp(pluginName, "TFAR", 4) == 0`). The
/// plugin also requires the command to come from the current server connection.
pub fn accepts_sender(plugin_name: impl AsRef<[u8]>) -> bool {
    plugin_name
        .as_ref()
        .starts_with(PLUGIN_NAME_PREFIX.as_bytes())
}
