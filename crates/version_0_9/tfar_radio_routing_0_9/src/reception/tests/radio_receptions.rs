//! Unit tests for `is_over_radio` and `receptions`, against `isOverRadio` in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:1091-1158`.

use super::*;
use tfar_client_state_0_9::prelude::{Position, UsableRadios, VehicleDescriptor};
use tfar_client_state_0_9::{FrequencySettings, SpeakerRadio};

const ALL_RADIOS: UsableRadios = UsableRadios {
    short_range: true,
    long_range: true,
    diver: true,
};

fn at(x: f64) -> Position {
    Position::from_array([x, 0.0, 0.0]).unwrap()
}

fn speaker(radio_id: &str, owner: &str, position: Option<Position>) -> SpeakerRadio {
    SpeakerRadio {
        radio_id: radio_id.to_owned(),
        owner_nickname: owner.to_owned(),
        position,
        volume: 8,
        vehicle: VehicleDescriptor {
            vehicle_name: "1:9".to_owned(),
            isolation: 0.4,
        },
        wave_z: -2.0,
    }
}

/// The listener `Me` (client 1) has `30.1` on short range; `Alpha` (client 2) holds a
/// short-range push-to-talk on `30.1` at 100 m with 1000 m range; `Bravo` (client 3) is current
/// at 40 m, `Stale` (client 4) is not.
fn server() -> ServerRadioData {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.current_data_frame = 3;
    server.sw_frequencies.insert(
        "30.1".to_owned(),
        FrequencySettings {
            volume: 7,
            stereo_mode: StereoMode::Left,
        },
    );
    for (nickname, id, x, frame) in [
        ("Me", 1, 0.0, 3),
        ("Alpha", 2, 100.0, 3),
        ("Bravo", 3, 40.0, 3),
        ("Stale", 4, 60.0, 0),
    ] {
        let mut client = ClientData::new(ClientId(id));
        client.usable_radios = ALL_RADIOS;
        client.position = at(x);
        client.data_frame = frame;
        server.clients.insert(nickname.to_owned(), client);
    }
    let alpha = server.client_mut("Alpha").unwrap();
    alpha.tangent = Some(TangentKind::ShortRange);
    alpha.frequency = "30.1".to_owned();
    alpha.range = 1000;
    server
}

fn heard_by_me(server: &ServerRadioData, ignore: IgnoreTangents) -> Vec<HeardVia> {
    receptions(server, "Alpha", ClientId(1), ignore)
}

fn summary(heard: &[HeardVia]) -> Vec<(HeardOn, String)> {
    heard
        .iter()
        .map(|heard| (heard.on, heard.radio_id.clone()))
        .collect()
}

#[test]
fn the_local_reception_comes_first_then_the_loudspeakers_in_the_games_order() {
    let mut server = server();
    server.speakers.insert(
        "30.1".to_owned(),
        vec![
            speaker("r1", "Bravo", Some(at(5.0))),
            speaker("r2", "Bravo", Some(at(6.0))),
        ],
    );
    assert_eq!(
        summary(&heard_by_me(&server, IgnoreTangents::NONE)),
        [
            (HeardOn::ShortRange, "local_radio".to_owned()),
            (HeardOn::Loudspeaker, "r1".to_owned()),
            (HeardOn::Loudspeaker, "r2".to_owned()),
        ]
    );
}

#[test]
fn a_loudspeaker_reception_carries_the_speakers_values() {
    let mut server = server();
    server.speakers.insert(
        "30.1".to_owned(),
        vec![speaker("r1", "Bravo", Some(at(5.0)))],
    );
    let heard = heard_by_me(&server, IgnoreTangents::NONE);
    let loudspeaker = &heard[1];
    assert_eq!(loudspeaker.over, TangentKind::ShortRange);
    assert_eq!(loudspeaker.volume, 8);
    assert_eq!(loudspeaker.stereo_mode, StereoMode::Both);
    assert_eq!(loudspeaker.position, at(5.0));
    assert_eq!(loudspeaker.vehicle.vehicle_name, "1:9");
    assert!(loudspeaker.is_under_water());
}

#[test]
fn a_speaker_on_frequency_owned_by_another_is_heard_and_the_senders_own_is_not() {
    let mut server = server();
    server.speakers.insert(
        "30.1".to_owned(),
        vec![
            speaker("own", "Alpha", Some(at(1.0))),
            speaker("other", "Bravo", Some(at(2.0))),
        ],
    );
    let ids: Vec<_> = heard_by_me(&server, IgnoreTangents::NONE)
        .into_iter()
        .filter(|heard| heard.on == HeardOn::Loudspeaker)
        .map(|heard| heard.radio_id)
        .collect();
    assert_eq!(ids, ["other"]);
}

#[test]
fn a_carried_speaker_takes_its_owners_position_while_the_owner_is_current() {
    let mut server = server();
    server.speakers.insert(
        "30.1".to_owned(),
        vec![
            speaker("carried", "Bravo", None),
            speaker("stale_owner", "Stale", None),
            speaker("unknown_owner", "Nobody", None),
        ],
    );
    let heard = heard_by_me(&server, IgnoreTangents::NONE);
    assert_eq!(
        summary(&heard),
        [
            (HeardOn::ShortRange, "local_radio".to_owned()),
            (HeardOn::Loudspeaker, "carried".to_owned()),
        ]
    );
    assert_eq!(heard[1].position, at(40.0));
}

#[test]
fn no_loudspeaker_carries_a_sender_out_of_range_or_without_a_held_radio() {
    let mut server = server();
    server.speakers.insert(
        "30.1".to_owned(),
        vec![speaker("r1", "Bravo", Some(at(5.0)))],
    );
    server.client_mut("Alpha").unwrap().range = 100;
    assert_eq!(heard_by_me(&server, IgnoreTangents::NONE).len(), 0);
    server.client_mut("Alpha").unwrap().range = 1000;
    server.client_mut("Alpha").unwrap().tangent = None;
    assert_eq!(heard_by_me(&server, IgnoreTangents::NONE).len(), 0);
    server.client_mut("Alpha").unwrap().tangent = Some(TangentKind::DiverDigital);
    assert!(
        heard_by_me(&server, IgnoreTangents::NONE).is_empty(),
        "a diver radio reaches no loudspeaker"
    );
}

#[test]
fn a_released_push_to_talk_reaches_the_loudspeakers_with_its_ignore_flag() {
    let mut server = server();
    server.speakers.insert(
        "30.1".to_owned(),
        vec![speaker("r1", "Bravo", Some(at(5.0)))],
    );
    server.client_mut("Alpha").unwrap().tangent = None;
    let heard = heard_by_me(&server, IgnoreTangents::for_kind(TangentKind::LongRange));
    assert_eq!(
        summary(&heard),
        [
            (HeardOn::ShortRange, "local_radio".to_owned()),
            (HeardOn::Loudspeaker, "r1".to_owned()),
        ]
    );
    assert!(
        heard
            .iter()
            .all(|heard| heard.over == TangentKind::LongRange)
    );
    let heard = heard_by_me(&server, IgnoreTangents::for_kind(TangentKind::DiverDigital));
    assert!(heard.is_empty(), "the diver flag reaches neither");
}

#[test]
fn a_long_range_sender_reaches_loudspeakers_over_long_range() {
    let mut server = server();
    server.speakers.insert(
        "30.1".to_owned(),
        vec![speaker("r1", "Bravo", Some(at(5.0)))],
    );
    server.client_mut("Alpha").unwrap().tangent = Some(TangentKind::LongRange);
    let heard = heard_by_me(&server, IgnoreTangents::NONE);
    assert!(
        heard
            .iter()
            .all(|heard| heard.over == TangentKind::LongRange)
    );
    assert_eq!(heard.len(), 2);
}

#[test]
fn the_listener_never_hears_their_own_transmission_on_their_own_radio() {
    let mut server = server();
    let me = server.client_mut("Me").unwrap();
    me.tangent = Some(TangentKind::ShortRange);
    me.frequency = "30.1".to_owned();
    me.range = 1000;
    server.speakers.insert(
        "30.1".to_owned(),
        vec![speaker("r1", "Bravo", Some(at(5.0)))],
    );
    let heard = receptions(&server, "Me", ClientId(1), IgnoreTangents::NONE);
    assert_eq!(summary(&heard), [(HeardOn::Loudspeaker, "r1".to_owned())]);
}

#[test]
fn a_missing_sender_or_a_listener_without_current_data_hears_nothing() {
    let mut server = server();
    assert_eq!(
        receptions(&server, "Nobody", ClientId(1), IgnoreTangents::NONE).len(),
        0
    );
    server.client_mut("Me").unwrap().data_frame = 0;
    assert_eq!(heard_by_me(&server, IgnoreTangents::NONE).len(), 0);
}
