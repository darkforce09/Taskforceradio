//! Serious mode and the channels of other radio plugins, by channel name.

#[cfg(test)]
#[path = "tests/serious_mode.rs"]
mod tests;

/// The channel names other Arma radio plugins use; in them TFAR leaves every mute alone
/// (`isOtherRadioPluginEnabled`, `reference/tfar_0_9_12/ts/src/plugin.cpp:896`).
pub const OTHER_RADIO_PLUGIN_CHANNELS: [&str; 7] = [
    "RT",
    "RT_A2",
    "RT_OFP",
    "RT_A3",
    "Radio",
    "[ARMA3.RU] Radio",
    "PvP_WOG",
];

/// Whether `channel_name` is one of [`OTHER_RADIO_PLUGIN_CHANNELS`], compared exactly.
pub fn is_other_radio_plugin_channel(channel_name: &str) -> bool {
    OTHER_RADIO_PLUGIN_CHANNELS.contains(&channel_name)
}

/// Whether a client in the channel `channel_name_of_client` is in serious mode: the mission names
/// a serious channel (`configured_name` is not empty) and the client sits in it
/// (`isSeriousModeEnabled`, plugin.cpp:885).
pub fn is_serious_mode(channel_name_of_client: &str, configured_name: &str) -> bool {
    !configured_name.is_empty() && channel_name_of_client == configured_name
}
