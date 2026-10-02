//! Which TeamSpeak plugins a 0.9.12 plugin takes commands from.

#[cfg(test)]
#[path = "tests/sender_filter.rs"]
mod tests;

/// The prefix every TFAR plugin's name starts with: `PLUGIN_NAME` in
/// `reference/tfar_0_9_12/ts/src/plugin.cpp`. The Windows builds register as
/// `task_force_radio_win32` and `task_force_radio_win64`, and TeamSpeak reports a sender by the
/// name its plugin file carries.
pub const PLUGIN_NAME_PREFIX: &str = "task_force_radio";

/// Whether a plugin command from the plugin named `plugin_name` is one of ours: its name starts
/// with [`PLUGIN_NAME_PREFIX`], the `strncmp(pluginName, PLUGIN_NAME, strlen(PLUGIN_NAME)) == 0`
/// test of `ts3plugin_onPluginCommandEvent` (plugin.cpp:3116). The comparison is on bytes and
/// case-sensitive.
pub fn accepts_sender(plugin_name: &str) -> bool {
    plugin_name
        .as_bytes()
        .starts_with(PLUGIN_NAME_PREFIX.as_bytes())
}
