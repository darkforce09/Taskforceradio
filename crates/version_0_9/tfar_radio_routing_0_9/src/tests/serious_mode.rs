//! Unit tests for the serious-mode channel checks, against `isSeriousModeEnabled` and
//! `isOtherRadioPluginEnabled` in `reference/tfar_0_9_12/ts/src/plugin.cpp:885-906`.

use super::*;

#[test]
fn every_other_radio_plugin_channel_is_recognised_exactly() {
    for name in OTHER_RADIO_PLUGIN_CHANNELS {
        assert!(is_other_radio_plugin_channel(name), "{name}");
    }
    assert!(!is_other_radio_plugin_channel("radio"));
    assert!(!is_other_radio_plugin_channel("RT "));
    assert!(!is_other_radio_plugin_channel("Lobby"));
}

#[test]
fn serious_mode_needs_a_configured_channel_and_the_client_in_it() {
    assert!(is_serious_mode("TFAR serious", "TFAR serious"));
    assert!(!is_serious_mode("Lobby", "TFAR serious"));
    assert!(
        !is_serious_mode("", ""),
        "no configured channel, no serious mode"
    );
}
