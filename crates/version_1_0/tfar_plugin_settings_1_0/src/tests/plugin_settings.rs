//! Unit tests for `PluginSettings`: `SETCFG` semantics, the refresh flag and the typed getters.

use super::*;
use crate::setting_value::SettingKind;

fn settings() -> PluginSettings {
    PluginSettings::default()
}

#[test]
fn every_setting_starts_at_its_default() {
    let settings = settings();
    for key in SettingKey::ALL {
        assert_eq!(
            settings.value(key),
            key.default_value(),
            "{}",
            key.wire_name()
        );
    }
}

#[test]
fn the_typed_getters_read_the_defaults() {
    let settings = settings();
    assert!(settings.full_duplex());
    assert_eq!(settings.addon_version(), "unknown");
    assert_eq!(settings.serious_channel_name(), "");
    assert_eq!(settings.serious_channel_password(), "");
    assert!((settings.intercom_volume() - 0.1).abs() < f32::EPSILON);
    assert!(settings.intercom_enabled());
    assert_eq!(settings.plugin_timeout(), Duration::from_secs(4));
    assert!(!settings.headset_lowered());
    assert!(!settings.spectator_not_hear_enemies());
    assert!(settings.spectator_can_hear_friendlies());
    assert_eq!(settings.tangent_release_delay(), Duration::ZERO);
    assert!(!settings.move_while_tabbed_out());
    assert!((settings.intercom_ducking() - 0.2).abs() < f32::EPSILON);
    assert!(settings.minimum_plugin_version().abs() < f32::EPSILON);
    assert!((settings.object_interception_strength() - 400.0).abs() < f32::EPSILON);
    assert!(settings.voice_cone());
    assert!(settings.allow_debugging());
    assert!(!settings.no_automove_spectator());
    assert!(!settings.disable_automatic_mute());
    assert!(!settings.mute_spectators());
}

#[test]
fn the_refresh_flag_starts_set() {
    // reference/tfar_1_0/ts/src/settings.hpp:215.
    assert!(settings().needs_refresh());
}

#[test]
fn any_successful_set_clears_the_refresh_flag() {
    // reference/tfar_1_0/ts/src/settings.hpp:152.
    let mut settings = settings();
    settings
        .set("voiceCone", "true", Some(TypeTag::Bool))
        .unwrap();
    assert!(!settings.needs_refresh());
}

#[test]
fn mark_needs_refresh_sets_the_flag_again() {
    // reference/tfar_1_0/ts/src/settings.hpp:187, called from task_force_radio.cpp:24.
    let mut settings = settings();
    settings
        .set("voiceCone", "true", Some(TypeTag::Bool))
        .unwrap();
    settings.mark_needs_refresh();
    assert!(settings.needs_refresh());
}

#[test]
fn a_bool_tag_sets_a_boolean_setting() {
    let mut settings = settings();
    let changed = settings
        .set("full_duplex", "false", Some(TypeTag::Bool))
        .unwrap();
    assert_eq!(changed.key, SettingKey::FullDuplex);
    assert!(!settings.full_duplex());
    settings
        .set("full_duplex", "TRUE", Some(TypeTag::Bool))
        .unwrap();
    assert!(settings.full_duplex());
}

#[test]
fn a_scalar_tag_sets_a_number_setting() {
    let mut settings = settings();
    settings
        .set("minimumPluginVersion", "339", Some(TypeTag::Scalar))
        .unwrap();
    assert!((settings.minimum_plugin_version() - 339.0).abs() < f32::EPSILON);
    settings
        .set("pluginTimeout", "2.5", Some(TypeTag::Scalar))
        .unwrap();
    assert_eq!(settings.plugin_timeout(), Duration::from_millis(2500));
}

#[test]
fn a_string_tag_sets_a_text_setting() {
    let mut settings = settings();
    settings
        .set("serious_channelName", "TaskForceRadio", Some(TypeTag::Text))
        .unwrap();
    settings
        .set("serious_channelPassword", "123", Some(TypeTag::Text))
        .unwrap();
    assert_eq!(settings.serious_channel_name(), "TaskForceRadio");
    assert_eq!(settings.serious_channel_password(), "123");
}

#[test]
fn a_number_set_on_a_boolean_setting_is_true_above_zero() {
    let mut settings = settings();
    settings
        .set("voiceCone", "0", Some(TypeTag::Scalar))
        .unwrap();
    assert!(!settings.voice_cone());
    settings
        .set("voiceCone", "0.25", Some(TypeTag::Scalar))
        .unwrap();
    assert!(settings.voice_cone());
}

#[test]
fn a_boolean_set_on_a_number_setting_is_one_or_zero() {
    let mut settings = settings();
    settings
        .set("intercomVolume", "true", Some(TypeTag::Bool))
        .unwrap();
    assert!((settings.intercom_volume() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn a_number_set_on_a_text_setting_is_its_six_decimal_text() {
    let mut settings = settings();
    settings
        .set("addon_version", "339", Some(TypeTag::Scalar))
        .unwrap();
    assert_eq!(settings.addon_version(), "339.000000");
}

#[test]
fn the_key_matches_by_case_insensitive_prefix() {
    let mut settings = settings();
    let changed = settings
        .set("SERIOUS_CHANNELNAMEextra", "Radio", Some(TypeTag::Text))
        .unwrap();
    assert_eq!(changed.key, SettingKey::SeriousChannelName);
    assert_eq!(settings.serious_channel_name(), "Radio");
}

#[test]
fn an_unknown_key_is_an_error_with_the_original_message() {
    // reference/tfar_1_0/ts/src/CommandProcessor.cpp:395-397.
    let mut settings = settings();
    let error = settings
        .set("someNewSetting", "1", Some(TypeTag::Scalar))
        .unwrap_err();
    assert_eq!(
        error,
        Error::UnknownKey {
            key: "someNewSetting".to_owned()
        }
    );
    assert_eq!(
        error.to_string(),
        "Used invalid config key: someNewSetting (You probably need to update your TFAR \
         Teamspeak plugin)"
    );
}

#[test]
fn a_failed_set_changes_nothing_and_keeps_the_refresh_flag() {
    let mut settings = settings();
    let before = settings.clone();
    assert!(
        settings
            .set("unknown_key", "x", Some(TypeTag::Text))
            .is_err()
    );
    assert!(
        settings
            .set("pluginTimeout", "soon", Some(TypeTag::Scalar))
            .is_err()
    );
    assert!(
        settings
            .set("voiceCone", "yes", Some(TypeTag::Text))
            .is_err()
    );
    assert_eq!(settings, before);
    assert!(settings.needs_refresh());
}

#[test]
fn a_wrong_token_count_sets_the_empty_text() {
    // tokens.size() != 4 -> set(key, ""), reference/tfar_1_0/ts/src/CommandProcessor.cpp:411-413.
    let mut settings = settings();
    settings
        .set("serious_channelName", "Radio", Some(TypeTag::Text))
        .unwrap();
    let changed = settings
        .set("serious_channelName", "Ignored", None)
        .unwrap();
    assert_eq!(changed.key, SettingKey::SeriousChannelName);
    assert_eq!(settings.serious_channel_name(), "");
}

#[test]
fn the_empty_text_fallback_on_a_typed_setting_is_an_error_not_a_crash() {
    // The original stores the text through the union pointer of a bool or float setting
    // (settings.hpp:96-98), which is undefined behaviour; here it is an error.
    let mut settings = settings();
    let error = settings.set("full_duplex", "true", None).unwrap_err();
    assert_eq!(
        error,
        Error::TextForTypedSetting {
            key: SettingKey::FullDuplex,
            value: String::new()
        }
    );
    assert!(settings.full_duplex());
    assert_eq!(SettingKey::FullDuplex.kind(), SettingKind::Bool);
}

#[test]
fn a_text_set_on_a_number_setting_is_an_error() {
    let mut settings = settings();
    let error = settings
        .set("pluginTimeout", "4", Some(TypeTag::Text))
        .unwrap_err();
    assert_eq!(
        error,
        Error::TextForTypedSetting {
            key: SettingKey::PluginTimeout,
            value: "4".to_owned()
        }
    );
}

#[test]
fn plugin_timeout_cuts_to_whole_milliseconds_and_never_goes_negative() {
    let mut settings = settings();
    settings
        .set("pluginTimeout", "1.0006", Some(TypeTag::Scalar))
        .unwrap();
    assert_eq!(settings.plugin_timeout(), Duration::from_millis(1000));
    settings
        .set("pluginTimeout", "-3", Some(TypeTag::Scalar))
        .unwrap();
    assert_eq!(settings.plugin_timeout(), Duration::ZERO);
}

#[test]
fn tangent_release_delay_is_whole_milliseconds() {
    // static_cast<int> of the float, reference/tfar_1_0/ts/src/CommandProcessor.cpp:364.
    let mut settings = settings();
    settings
        .set("tangentReleaseDelay", "250.9", Some(TypeTag::Scalar))
        .unwrap();
    assert_eq!(settings.tangent_release_delay(), Duration::from_millis(250));
    settings
        .set("tangentReleaseDelay", "-50", Some(TypeTag::Scalar))
        .unwrap();
    assert_eq!(settings.tangent_release_delay(), Duration::ZERO);
}

#[test]
fn every_setting_can_be_set_through_its_wire_name() {
    let mut settings = settings();
    for key in SettingKey::ALL {
        let (value, tag) = match key.kind() {
            SettingKind::Bool => ("false", TypeTag::Bool),
            SettingKind::Number => ("7", TypeTag::Scalar),
            SettingKind::Text => ("value", TypeTag::Text),
        };
        let changed = settings.set(key.wire_name(), value, Some(tag)).unwrap();
        assert_eq!(changed.key, key);
        let expected = match key.kind() {
            SettingKind::Bool => SettingValue::Bool(false),
            SettingKind::Number => SettingValue::Number(7.0),
            SettingKind::Text => SettingValue::Text("value".to_owned()),
        };
        assert_eq!(settings.value(key), expected, "{}", key.wire_name());
    }
}

#[test]
fn the_addon_configuration_applies_cleanly() {
    // The SETCFG messages fnc_sendPluginConfig.sqf:21-41 sends with its defaults, formatted
    // as fnc_setPluginSetting.sqf:28 formats them.
    let messages = [
        ("minimumPluginVersion", "339", "SCALAR"),
        ("full_duplex", "true", "BOOL"),
        ("addon_version", "1.0.329", "STRING"),
        ("serious_channelPassword", "123", "STRING"),
        ("serious_channelName", "TaskForceRadio", "STRING"),
        ("intercomVolume", "0.3", "SCALAR"),
        ("intercomDucking", "0.2", "SCALAR"),
        ("intercomEnabled", "true", "BOOL"),
        ("pluginTimeout", "4", "SCALAR"),
        ("spectatorNotHearEnemies", "true", "BOOL"),
        ("spectatorCanHearFriendlies", "true", "BOOL"),
        ("tangentReleaseDelay", "0", "SCALAR"),
        ("headsetLowered", "false", "BOOL"),
        ("moveWhileTabbedOut", "false", "BOOL"),
        ("objectInterceptionStrength", "400", "SCALAR"),
        ("voiceCone", "true", "BOOL"),
        ("disableAutomaticMute", "false", "BOOL"),
        ("noAutomoveSpectator", "true", "BOOL"),
        ("allowDebugging", "false", "BOOL"),
        ("muteSpectators", "false", "BOOL"),
    ];
    let mut settings = settings();
    for (key, value, tag) in messages {
        settings
            .set(key, value, Some(TypeTag::from_wire(tag)))
            .unwrap();
    }
    assert_eq!(settings.addon_version(), "1.0.329");
    assert_eq!(settings.serious_channel_name(), "TaskForceRadio");
    assert!(settings.no_automove_spectator());
    assert!(!settings.allow_debugging());
    assert!((settings.intercom_volume() - 0.3).abs() < f32::EPSILON);
    assert!(!settings.needs_refresh());
}
