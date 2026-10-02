//! Unit tests for the incoming plugin commands, against `processTangentPress`,
//! `processAllTangentRelease` and the `VOLUME` branch in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:2940-3110`.

use std::time::Duration;

use super::*;
use crate::dsp_state::{DspRequirements, RadioEffectKind, RadioRequirement};

fn server() -> ServerRadioData {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.update_nicknames(&[
        (ClientId(1), "Me".to_owned()),
        (ClientId(2), "Alpha".to_owned()),
    ]);
    server.current_data_frame = 0;
    server.is_alive = true;
    server
}

fn tangent(kind: TangentKind, is_pressed: bool, subtype: &RadioSubtype) -> RemoteTangent<'_> {
    RemoteTangent {
        kind,
        is_pressed,
        frequency: "30.1_blufor",
        range: 3000,
        subtype,
        nickname: "Alpha",
    }
}

fn with_radio_effect(server: &mut ServerRadioData) {
    let requirements = DspRequirements {
        direct_voice: None,
        radios: vec![RadioRequirement {
            radio_id: "local_radio".to_owned(),
            effect: RadioEffectKind::Personal,
            speaker: None,
        }],
    };
    server
        .client_mut("Alpha")
        .unwrap()
        .ensure_dsp(&requirements, None)
        .unwrap();
}

#[test]
fn a_press_stores_the_transmission_and_asks_for_the_start_sound() {
    let mut server = server();
    let now = Instant::now();
    let outcome = server
        .apply_remote_tangent(
            &tangent(
                TangentKind::LongRange,
                true,
                &RadioSubtype::DigitalLongRange,
            ),
            now,
        )
        .unwrap();
    assert_eq!(
        outcome,
        RemoteTangentOutcome {
            client_id: ClientId(2),
            has_fresh_data: true,
            sound: Some(TangentSound::Start),
            is_listener_alive: true,
            dropped_radio_effects: false,
        }
    );
    let alpha = server.client("Alpha").unwrap();
    assert_eq!(alpha.tangent, Some(TangentKind::LongRange));
    assert_eq!(alpha.frequency, "30.1_blufor");
    assert_eq!(alpha.range, 3000);
    assert_eq!(alpha.subtype, RadioSubtype::DigitalLongRange);
    assert!(alpha.can_use(TangentKind::LongRange));
    assert!(alpha.is_plugin_enabled);
    assert_eq!(alpha.plugin_enabled_check, Some(now));
    assert_eq!(alpha.position_time, Some(now));
}

#[test]
fn a_repeated_press_asks_for_no_sound() {
    let mut server = server();
    let press = tangent(TangentKind::ShortRange, true, &RadioSubtype::Digital);
    server.apply_remote_tangent(&press, Instant::now());
    let outcome = server.apply_remote_tangent(&press, Instant::now()).unwrap();
    assert_eq!(outcome.sound, None);
}

#[test]
fn a_release_heard_alive_drops_the_senders_radio_effects() {
    let mut server = server();
    let subtype = RadioSubtype::Digital;
    server.apply_remote_tangent(
        &tangent(TangentKind::ShortRange, true, &subtype),
        Instant::now(),
    );
    with_radio_effect(&mut server);
    let outcome = server
        .apply_remote_tangent(
            &tangent(TangentKind::ShortRange, false, &subtype),
            Instant::now(),
        )
        .unwrap();
    assert_eq!(outcome.sound, Some(TangentSound::End));
    assert!(outcome.dropped_radio_effects);
    let alpha = server.client_mut("Alpha").unwrap();
    assert_eq!(alpha.tangent, None);
    assert!(alpha.dsp.radio_mut("local_radio").is_none());
}

#[test]
fn a_release_heard_dead_or_without_current_data_keeps_the_radio_effects() {
    let subtype = RadioSubtype::Digital;
    for (is_alive, frame) in [(false, 0), (true, 5)] {
        let mut server = server();
        server.is_alive = is_alive;
        server.apply_remote_tangent(
            &tangent(TangentKind::ShortRange, true, &subtype),
            Instant::now(),
        );
        with_radio_effect(&mut server);
        server.current_data_frame = frame;
        let outcome = server
            .apply_remote_tangent(
                &tangent(TangentKind::ShortRange, false, &subtype),
                Instant::now(),
            )
            .unwrap();
        assert!(
            !outcome.dropped_radio_effects,
            "alive {is_alive}, frame {frame}"
        );
        assert!(
            server
                .client_mut("Alpha")
                .unwrap()
                .dsp
                .radio_mut("local_radio")
                .is_some()
        );
    }
}

#[test]
fn a_tangent_from_an_unknown_nickname_changes_nothing() {
    let mut server = server();
    let subtype = RadioSubtype::Digital;
    let mut press = tangent(TangentKind::ShortRange, true, &subtype);
    press.nickname = "Nobody";
    assert_eq!(server.apply_remote_tangent(&press, Instant::now()), None);
    assert!(server.client("Nobody").is_none());
}

#[test]
fn release_all_tangents_ends_the_players_push_to_talk() {
    let mut server = server();
    server.apply_remote_tangent(
        &tangent(TangentKind::DiverDigital, true, &RadioSubtype::DiverDigital),
        Instant::now(),
    );
    assert!(server.apply_release_all_tangents("Alpha"));
    assert_eq!(server.client("Alpha").unwrap().tangent, None);
    assert!(!server.apply_release_all_tangents("Nobody"));
}

#[test]
fn volume_stores_the_voice_range_and_talking_state() {
    let mut server = server();
    let now = Instant::now() + Duration::from_secs(1);
    let outcome = server.apply_remote_volume("Alpha", 40, true, now).unwrap();
    assert_eq!(
        outcome,
        RemoteVolumeOutcome {
            client_id: ClientId(2),
            is_mine: false,
            has_fresh_data: true,
        }
    );
    let alpha = server.client("Alpha").unwrap();
    assert_eq!(alpha.voice_volume, 40);
    assert!(alpha.is_talking_now);
    assert!(alpha.is_plugin_enabled);
    assert_eq!(alpha.plugin_enabled_check, Some(now));
}

#[test]
fn volume_of_the_local_player_is_marked_mine() {
    let mut server = server();
    let outcome = server
        .apply_remote_volume("Me", 20, false, Instant::now())
        .unwrap();
    assert!(outcome.is_mine);
    assert_eq!(
        server.apply_remote_volume("Nobody", 20, false, Instant::now()),
        None
    );
}
