//! Unit tests for `SettingKey`: the wire names, the order, the defaults and the prefix key match.

use super::*;

#[test]
fn every_setting_is_listed_once_in_the_original_order() {
    // reference/tfar_1_0/ts/src/settings.hpp:12-31, in that order.
    let names: Vec<&str> = SettingKey::ALL.iter().map(|key| key.wire_name()).collect();
    assert_eq!(
        names,
        [
            "full_duplex",
            "addon_version",
            "serious_channelName",
            "serious_channelPassword",
            "intercomVolume",
            "intercomEnabled",
            "pluginTimeout",
            "headsetLowered",
            "spectatorNotHearEnemies",
            "spectatorCanHearFriendlies",
            "tangentReleaseDelay",
            "moveWhileTabbedOut",
            "intercomDucking",
            "minimumPluginVersion",
            "objectInterceptionStrength",
            "voiceCone",
            "allowDebugging",
            "noAutomoveSpectator",
            "disableAutomaticMute",
            "muteSpectators",
        ]
    );
}

#[test]
fn the_defaults_are_the_original_table() {
    // reference/tfar_1_0/ts/src/settings.hpp:12-31.
    let defaults: Vec<SettingValue> = SettingKey::ALL
        .iter()
        .map(|key| key.default_value())
        .collect();
    assert_eq!(
        defaults,
        [
            SettingValue::Bool(true),
            SettingValue::Text("unknown".to_owned()),
            SettingValue::Text(String::new()),
            SettingValue::Text(String::new()),
            SettingValue::Number(0.1),
            SettingValue::Bool(true),
            SettingValue::Number(4.0),
            SettingValue::Bool(false),
            SettingValue::Bool(false),
            SettingValue::Bool(true),
            SettingValue::Number(0.0),
            SettingValue::Bool(false),
            SettingValue::Number(0.2),
            SettingValue::Number(0.0),
            SettingValue::Number(400.0),
            SettingValue::Bool(true),
            SettingValue::Bool(true),
            SettingValue::Bool(false),
            SettingValue::Bool(false),
            SettingValue::Bool(false),
        ]
    );
}

#[test]
fn every_wire_name_matches_its_own_setting() {
    for key in SettingKey::ALL {
        assert_eq!(SettingKey::from_key_text(key.wire_name()), Some(key));
    }
}

#[test]
fn a_key_matches_in_any_letter_case() {
    // _strnicmp, reference/tfar_1_0/ts/src/settings.hpp:49.
    assert_eq!(
        SettingKey::from_key_text("SERIOUS_CHANNELNAME"),
        Some(SettingKey::SeriousChannelName)
    );
    assert_eq!(
        SettingKey::from_key_text("Full_Duplex"),
        Some(SettingKey::FullDuplex)
    );
}

#[test]
fn a_key_that_runs_past_a_setting_name_still_matches_it() {
    // The comparison runs over strlen(SettingStrings[i]) only, settings.hpp:49.
    assert_eq!(
        SettingKey::from_key_text("full_duplex_extra"),
        Some(SettingKey::FullDuplex)
    );
    assert_eq!(
        SettingKey::from_key_text("voiceConeAngle"),
        Some(SettingKey::VoiceCone)
    );
}

#[test]
fn a_key_shorter_than_every_name_matches_nothing() {
    assert_eq!(SettingKey::from_key_text("full_dupl"), None);
    assert_eq!(SettingKey::from_key_text(""), None);
}

#[test]
fn an_unknown_key_matches_nothing() {
    assert_eq!(SettingKey::from_key_text("someNewSetting"), None);
}

#[test]
fn no_wire_name_is_a_prefix_of_another_so_the_first_match_is_the_only_one() {
    for key in SettingKey::ALL {
        for other in SettingKey::ALL {
            if key != other {
                assert!(
                    !other
                        .wire_name()
                        .to_ascii_lowercase()
                        .starts_with(&key.wire_name().to_ascii_lowercase()),
                    "{} is a prefix of {}",
                    key.wire_name(),
                    other.wire_name()
                );
            }
        }
    }
}

#[test]
fn the_kind_of_a_setting_is_the_kind_of_its_default() {
    assert_eq!(SettingKey::FullDuplex.kind(), SettingKind::Bool);
    assert_eq!(SettingKey::AddonVersion.kind(), SettingKind::Text);
    assert_eq!(SettingKey::PluginTimeout.kind(), SettingKind::Number);
}
