//! The `ts3plugin_*` exports TeamSpeak resolves, written by `ts3_plugin!` over the one plugin
//! instance.

use crate::plugin_lifecycle::TfarPlugin;

/// The plugin instance behind every `ts3plugin_*` export.
pub static PLUGIN: TfarPlugin = TfarPlugin::new();

tfar_ts3_host::ts3_plugin!(PLUGIN);
