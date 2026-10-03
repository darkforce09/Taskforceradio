//! Unit tests for the local tangent, against the `TANGENT` and `RELEASE_ALL_TANGENTS` cases of
//! `reference/tfar_1_0/ts/src/CommandProcessor.cpp:293-376`.

use tfar_plugin_settings_1_0::prelude::TypeTag;

use super::*;
use crate::test_support::{frequency_report, session};

/// A session in a game with the radios of [`frequency_report`] on (short range `100` at volume
/// 7, long range `40` at volume 5, global volume 0.5); the local player can use short range.
fn tuned_session() -> SessionState {
    let mut session = session();
    session.apply_frequencies(&frequency_report("me"));
    session.clients.own_mut().unwrap().can_use_sw_radio = true;
    session
}

fn press(kind: GameTangentKind, is_pressed: bool, frequency: &str) -> TangentPress {
    TangentPress {
        kind,
        is_pressed,
        frequency: frequency.to_owned(),
        range: 20_000,
        range_text: "20000".to_owned(),
        subtype: RadioSubtype::Digital,
        radio_classname: Some("tf_anprc152_1".to_owned()),
    }
}

fn half_duplex() -> PluginSettings {
    let mut settings = PluginSettings::default();
    settings
        .set("full_duplex", "false", Some(TypeTag::from_wire("BOOL")))
        .unwrap();
    settings
}

#[test]
fn a_press_sets_the_own_transmit_state_and_asks_for_the_sound_and_broadcast() {
    let mut session = tuned_session();
    let applied = session
        .apply_local_tangent(
            &press(GameTangentKind::ShortRange, true, "100"),
            &PluginSettings::default(),
        )
        .unwrap();
    assert!(session.game.is_tangent_pressed);
    let own = session.clients.own().unwrap();
    assert_eq!(own.range, 20_000);
    assert_eq!(own.transmitting_over, Some(TangentRadio::ShortRange));
    assert_eq!(own.transmitting_frequency.as_deref(), Some("100"));
    assert_eq!(own.transmitting_subtype, Some(RadioSubtype::Digital));
    assert!(applied.is_pressed);
    let sound = applied.sound.unwrap();
    assert_eq!(sound.sound, "radio-sounds/sw/local_start");
    assert!((sound.gain - radio_volume_gain(7.0) * 0.5).abs() < 1e-12);
    assert_eq!(sound.stereo_mode, StereoMode::Both);
    assert_eq!(
        applied.broadcast,
        TangentBroadcast {
            kind: TangentKind::ShortRange,
            is_pressed: true,
            frequency: "100".to_owned(),
            range: 20_000,
            range_text: "20000".to_owned(),
            subtype: RadioSubtype::Digital,
            radio_classname: Some("tf_anprc152_1".to_owned()),
            can_use_sw_radio: true,
            can_use_dd_radio: false,
            nickname: "me".to_owned(),
        }
    );
}

#[test]
fn only_a_change_of_the_tangent_acts() {
    let mut session = tuned_session();
    let settings = PluginSettings::default();
    let first = press(GameTangentKind::ShortRange, true, "100");
    assert!(session.apply_local_tangent(&first, &settings).is_some());
    let mut repeated = press(GameTangentKind::LongRange, true, "40");
    repeated.range = 5;
    assert_eq!(session.apply_local_tangent(&repeated, &settings), None);
    let own = session.clients.own().unwrap();
    assert_eq!(own.range, 20_000, "a repeated press changes no state");
    assert_eq!(own.transmitting_frequency.as_deref(), Some("100"));
    let release = press(GameTangentKind::ShortRange, false, "100");
    assert!(session.apply_local_tangent(&release, &settings).is_some());
    assert_eq!(session.apply_local_tangent(&release, &settings), None);
}

#[test]
fn a_release_clears_the_transmit_state_and_keeps_the_previous_frequency() {
    let mut session = tuned_session();
    let settings = PluginSettings::default();
    session.apply_local_tangent(&press(GameTangentKind::ShortRange, true, "100"), &settings);
    let applied = session
        .apply_local_tangent(&press(GameTangentKind::ShortRange, false, "100"), &settings)
        .unwrap();
    assert!(!session.game.is_tangent_pressed);
    let own = session.clients.own().unwrap();
    assert_eq!(own.range, 0);
    assert_eq!(own.transmitting_over, None);
    assert_eq!(own.transmitting_frequency.as_deref(), Some("prev_100"));
    assert!(!applied.is_pressed);
    assert!(!applied.broadcast.is_pressed);
    assert_eq!(applied.sound.unwrap().sound, "radio-sounds/sw/local_end");
}

#[test]
fn a_long_range_tangent_transmits_on_long_range_and_a_diver_tangent_on_short_range() {
    let mut session = tuned_session();
    let settings = PluginSettings::default();
    session.apply_local_tangent(&press(GameTangentKind::LongRange, true, "40"), &settings);
    let own = session.clients.own().unwrap();
    assert_eq!(own.transmitting_over, Some(TangentRadio::LongRange));
    session.apply_local_tangent(&press(GameTangentKind::LongRange, false, "40"), &settings);
    let applied = session
        .apply_local_tangent(&press(GameTangentKind::DiverDigital, true, "40"), &settings)
        .unwrap();
    let own = session.clients.own().unwrap();
    assert_eq!(own.transmitting_over, Some(TangentRadio::ShortRange));
    assert_eq!(applied.broadcast.kind, TangentKind::DiverDigital);
}

#[test]
fn the_sound_level_comes_from_the_list_the_subtype_picks_or_is_full_when_not_found() {
    let mut session = tuned_session();
    let settings = PluginSettings::default();
    let mut long_range = press(GameTangentKind::LongRange, true, "40");
    long_range.subtype = RadioSubtype::DigitalLongRange;
    let sound = session
        .apply_local_tangent(&long_range, &settings)
        .unwrap()
        .sound
        .unwrap();
    assert_eq!(sound.sound, "radio-sounds/lr/local_start");
    assert!((sound.gain - radio_volume_gain(5.0) * 0.5).abs() < 1e-12);
    let mut unknown = press(GameTangentKind::ShortRange, false, "77");
    unknown.subtype = RadioSubtype::Airborne;
    let sound = session
        .apply_local_tangent(&unknown, &settings)
        .unwrap()
        .sound
        .unwrap();
    assert_eq!(sound.sound, "radio-sounds/ab/local_end");
    assert!((sound.gain - 1.0).abs() < 1e-12);
    assert_eq!(sound.stereo_mode, StereoMode::Both);
}

#[test]
fn a_radio_without_a_local_sound_still_changes_the_state() {
    let mut session = tuned_session();
    let mut diver = press(GameTangentKind::DiverDigital, true, "100");
    diver.subtype = RadioSubtype::DiverDigital;
    let applied = session
        .apply_local_tangent(&diver, &PluginSettings::default())
        .unwrap();
    assert_eq!(applied.sound, None);
    assert!(session.game.is_tangent_pressed);
}

#[test]
fn half_duplex_tracks_the_transmitting_radio_and_full_duplex_never_does() {
    let mut session = tuned_session();
    let settings = half_duplex();
    session.apply_local_tangent(&press(GameTangentKind::ShortRange, true, "100"), &settings);
    assert_eq!(
        session.game.current_transmitting_radio.as_deref(),
        Some("tf_anprc152_1")
    );
    session.apply_local_tangent(&press(GameTangentKind::ShortRange, false, "100"), &settings);
    assert_eq!(session.game.current_transmitting_radio, None);
    session.game.current_transmitting_radio = Some("tf_rt1523g".to_owned());
    let mut without_class = press(GameTangentKind::ShortRange, true, "100");
    without_class.radio_classname = None;
    session.apply_local_tangent(&without_class, &settings);
    assert_eq!(
        session.game.current_transmitting_radio.as_deref(),
        Some("tf_rt1523g"),
        "a press without a class name leaves the radio as it is"
    );
    let mut full_duplex = tuned_session();
    full_duplex.apply_local_tangent(
        &press(GameTangentKind::ShortRange, true, "100"),
        &PluginSettings::default(),
    );
    assert_eq!(full_duplex.game.current_transmitting_radio, None);
}

#[test]
fn without_a_local_player_a_tangent_changes_nothing() {
    let mut session = SessionState::new();
    let applied = session.apply_local_tangent(
        &press(GameTangentKind::ShortRange, true, "100"),
        &PluginSettings::default(),
    );
    assert_eq!(applied, None);
    assert!(!session.game.is_tangent_pressed);
}

#[test]
fn release_all_tangents_clears_the_tangent_so_the_next_press_acts() {
    let mut session = tuned_session();
    let settings = half_duplex();
    let first = press(GameTangentKind::ShortRange, true, "100");
    session.apply_local_tangent(&first, &settings);
    let released = session.apply_local_release_all_tangents();
    assert!(released.was_tangent_pressed);
    assert!(!session.game.is_tangent_pressed);
    assert_eq!(session.game.current_transmitting_radio, None);
    let own = session.clients.own().unwrap();
    assert_eq!(own.range, 0);
    assert_eq!(own.transmitting_over, None);
    let again = session.apply_local_tangent(&first, &settings).unwrap();
    assert!(again.is_pressed, "the press after a release-all transmits");
    assert!(
        session
            .apply_local_release_all_tangents()
            .was_tangent_pressed
    );
    assert!(
        !session
            .apply_local_release_all_tangents()
            .was_tangent_pressed
    );
}
