//! Unit tests for `ClientData`, against `CLIENT_DATA` in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:192-366` and `isPluginEnabledForUser` (plugin.cpp:2526).

use super::*;

#[test]
fn a_new_entry_has_the_original_defaults() {
    let client = ClientData::new(ClientId(7));
    assert_eq!(client.client_id, ClientId(7));
    assert!(client.can_speak);
    assert_eq!(client.tangent, None);
    assert_eq!(client.position, Position::ORIGIN);
    assert_eq!(client.position_time, None);
    assert_eq!(client.data_frame, 0);
    assert_eq!(client.voice_volume, 0);
    assert_eq!(client.range, 0);
    assert_eq!(client.terrain_interception, 0);
    assert!(!client.is_talking_now);
    assert!(!client.can_use(TangentKind::ShortRange));
    assert!(!client.can_use(TangentKind::LongRange));
    assert!(!client.can_use(TangentKind::DiverDigital));
    assert!((client.voice_volume_multiplier - 1.0).abs() < 1e-12);
    assert!(client.dsp.holds_no_voice_state());
}

#[test]
fn fields_the_original_leaves_uninitialised_start_at_defined_defaults() {
    // `pluginEnabled`, `pluginEnabledCheck` and `viewAngle` are never set by the C++ constructor.
    let mut client = ClientData::new(ClientId(1));
    assert!(!client.is_plugin_enabled);
    assert_eq!(client.plugin_enabled_check, None);
    assert_eq!(client.view_angle, Angle::ZERO);
    // With no sign ever seen, a failed metadata check gives no grace.
    assert!(!client.refresh_plugin_enabled(false, Instant::now()));
}

#[test]
fn a_positive_metadata_check_enables_and_stamps() {
    let mut client = ClientData::new(ClientId(1));
    let now = Instant::now();
    assert!(client.refresh_plugin_enabled(true, now));
    assert!(client.is_plugin_enabled);
    assert_eq!(client.plugin_enabled_check, Some(now));
}

#[test]
fn a_failed_check_keeps_the_last_state_within_ten_seconds_and_drops_it_after() {
    let mut client = ClientData::new(ClientId(1));
    let start = Instant::now();
    client.refresh_plugin_enabled(true, start);
    let within = start + Duration::from_millis(9_999);
    assert!(client.refresh_plugin_enabled(false, within));
    assert!(!client.refresh_plugin_enabled(false, start + PLUGIN_ENABLED_GRACE));
    assert!(!client.is_plugin_enabled);
    // Once dropped, a later failed check inside the window stays dropped.
    assert!(!client.refresh_plugin_enabled(false, within));
}

#[test]
fn a_held_tangent_or_a_talking_volume_counts_as_talking_on_radio() {
    let mut client = ClientData::new(ClientId(1));
    assert!(!client.is_talking_on_radio());
    client.tangent = Some(TangentKind::LongRange);
    assert!(client.is_talking_on_radio());
    client.tangent = None;
    client.is_talking_now = true;
    assert!(client.is_talking_on_radio());
}

#[test]
fn allowing_a_radio_family_lets_the_player_use_only_that_family() {
    let mut client = ClientData::new(ClientId(1));
    client.allow_radio(TangentKind::DiverDigital);
    assert!(client.can_use(TangentKind::DiverDigital));
    assert!(!client.can_use(TangentKind::ShortRange));
    assert!(!client.can_use(TangentKind::LongRange));
}
