//! Table-driven tests of `is_over_radio`: intercom, range, speakers and towers.

use super::*;
use crate::fixtures::{ALPHA, BRAVO, SW_FREQUENCY, World, at};
use tfar_client_state_1_0::prelude::{ReceivedOn, Speaker, TangentRadio, VehicleDescriptor};
use tfar_geometry::Position;

fn heard(world: &World) -> Vec<ReceivedOn> {
    is_over_radio(
        &world.context(),
        world.sender("alpha"),
        IgnoreTangents::NONE,
    )
    .into_iter()
    .map(|via| via.on)
    .collect()
}

fn in_vehicle(world: &mut World, nickname: &str, slot: i16) {
    world.client(nickname).vehicle = VehicleDescriptor {
        name: "2:390".to_owned(),
        isolation: 0.5,
        intercom_slot: slot,
        velocity: [0.0; 3],
    };
}

fn speaker(
    owner: Option<tfar_client_state_1_0::prelude::TeamspeakClientId>,
    position: Option<Position>,
) -> Speaker {
    Speaker {
        radio_id: "2:500".to_owned(),
        owner,
        position,
        volume: 8.0,
        vehicle: VehicleDescriptor::not_in_vehicle(),
        wave_z: 1.0,
    }
}

fn with_speakers(world: &mut World, frequency: &str, speakers: Vec<Speaker>) {
    world
        .session
        .game
        .speakers
        .insert(frequency.to_owned(), speakers);
}

#[test]
fn the_intercom_carries_direct_speech_in_the_same_vehicle_and_slot() {
    let mut world = World::new();
    in_vehicle(&mut world, "alpha", 1);
    world.session.clients.own_mut().unwrap().vehicle = world.sender("alpha").vehicle.clone();
    assert_eq!(heard(&world), [ReceivedOn::Intercom]);
    let via = is_over_radio(
        &world.context(),
        world.sender("alpha"),
        IgnoreTangents::NONE,
    );
    assert_eq!(via.first().unwrap().radio_id, "intercom");
}

#[test]
fn the_intercom_case_table() {
    type Prepare = fn(&mut World);
    let cases: [(&str, Prepare, &[ReceivedOn]); 4] = [
        ("another slot", |world| in_vehicle(world, "alpha", 2), &[]),
        (
            "no intercom slot",
            |world| {
                in_vehicle(world, "alpha", -1);
                world
                    .session
                    .clients
                    .own_mut()
                    .unwrap()
                    .vehicle
                    .intercom_slot = -1;
            },
            &[],
        ),
        (
            "intercom disabled",
            |world| world.set_setting("intercomEnabled", "false"),
            &[],
        ),
        (
            "a sender on a radio talks on the radio, not the intercom",
            |world| {
                world.transmit(
                    "alpha",
                    TangentRadio::ShortRange,
                    "999",
                    3000,
                    Position::ORIGIN,
                );
            },
            &[],
        ),
    ];
    for (name, prepare, expected) in cases {
        let mut world = World::new();
        in_vehicle(&mut world, "alpha", 1);
        in_vehicle(&mut world, "me", 1);
        prepare(&mut world);
        assert_eq!(heard(&world), expected, "{name}");
    }
}

#[test]
fn a_sender_without_range_is_heard_on_no_radio() {
    let mut world = World::new();
    world.transmit(
        "alpha",
        TangentRadio::ShortRange,
        SW_FREQUENCY,
        0,
        Position::ORIGIN,
    );
    assert_eq!(heard(&world), []);
}

#[test]
fn the_listener_never_hears_itself_on_its_own_radio() {
    let mut world = World::new();
    world.transmit(
        "me",
        TangentRadio::ShortRange,
        SW_FREQUENCY,
        3000,
        Position::ORIGIN,
    );
    let own = world.session.clients.own().unwrap();
    assert_eq!(
        is_over_radio(&world.context(), own, IgnoreTangents::NONE),
        []
    );
}

#[test]
fn a_speaker_on_the_exact_frequency_plays_the_transmission() {
    let mut world = World::new();
    world.transmit(
        "alpha",
        TangentRadio::ShortRange,
        SW_FREQUENCY,
        3000,
        at(100.0, 0.0, 0.0),
    );
    with_speakers(
        &mut world,
        SW_FREQUENCY,
        vec![speaker(Some(BRAVO), Some(at(5.0, 5.0, 0.0)))],
    );
    with_speakers(
        &mut world,
        "100.0",
        vec![speaker(Some(BRAVO), Some(at(5.0, 5.0, 0.0)))],
    );
    assert_eq!(heard(&world), [ReceivedOn::ShortRange, ReceivedOn::Ground]);
    let via = is_over_radio(
        &world.context(),
        world.sender("alpha"),
        IgnoreTangents::NONE,
    );
    let ground = via.get(1).unwrap();
    assert_eq!(ground.position, Some(at(5.0, 5.0, 0.0)));
    assert_eq!(ground.radio_id, "2:500");
    assert!((ground.volume - 8.0).abs() < f64::EPSILON);
}

#[test]
fn the_speaker_case_table() {
    type Prepare = fn(&mut World);
    let cases: [(&str, Prepare, &[ReceivedOn]); 6] = [
        (
            "the sender's own speaker is skipped",
            |world| {
                with_speakers(
                    world,
                    SW_FREQUENCY,
                    vec![speaker(Some(ALPHA), Some(at(5.0, 0.0, 0.0)))],
                );
            },
            &[ReceivedOn::ShortRange],
        ),
        (
            "a speaker under water is skipped",
            |world| {
                with_speakers(
                    world,
                    SW_FREQUENCY,
                    vec![speaker(Some(BRAVO), Some(at(5.0, 0.0, -0.5)))],
                );
            },
            &[ReceivedOn::ShortRange],
        ),
        (
            "a speaker at the water line plays",
            |world| {
                with_speakers(
                    world,
                    SW_FREQUENCY,
                    vec![speaker(Some(BRAVO), Some(at(5.0, 0.0, 0.0)))],
                );
            },
            &[ReceivedOn::ShortRange, ReceivedOn::Ground],
        ),
        (
            "a speaker without position or known owner is skipped",
            |world| {
                with_speakers(world, SW_FREQUENCY, vec![speaker(None, None)]);
            },
            &[ReceivedOn::ShortRange],
        ),
        (
            "a speaker on its owner plays at the owner",
            |world| {
                world.client("bravo").position = at(7.0, 0.0, 0.0);
                with_speakers(world, SW_FREQUENCY, vec![speaker(Some(BRAVO), None)]);
            },
            &[ReceivedOn::ShortRange, ReceivedOn::Ground],
        ),
        (
            "a sender without a usable radio plays on no speaker",
            |world| {
                world.client("alpha").can_use_sw_radio = false;
                with_speakers(
                    world,
                    SW_FREQUENCY,
                    vec![speaker(Some(BRAVO), Some(at(5.0, 0.0, 0.0)))],
                );
            },
            &[],
        ),
    ];
    for (name, prepare, expected) in cases {
        let mut world = World::new();
        world.transmit(
            "alpha",
            TangentRadio::ShortRange,
            SW_FREQUENCY,
            3000,
            at(100.0, 0.0, 0.0),
        );
        prepare(&mut world);
        assert_eq!(heard(&world), expected, "{name}");
    }
}

#[test]
fn speakers_beyond_range_play_only_through_a_tower() {
    let mut world = World::new();
    world.transmit(
        "alpha",
        TangentRadio::ShortRange,
        "777",
        3000,
        at(5000.0, 0.0, 0.0),
    );
    with_speakers(
        &mut world,
        "777",
        vec![speaker(Some(BRAVO), Some(at(5.0, 0.0, 0.0)))],
    );
    assert_eq!(heard(&world), []);
    world.add_tower(1, at(4000.0, 0.0, 0.0), 10000.0);
    let via = is_over_radio(
        &world.context(),
        world.sender("alpha"),
        IgnoreTangents::NONE,
    );
    let ground = via.first().unwrap();
    assert_eq!(ground.on, ReceivedOn::Ground);
    let expected = 4000.0 / 10000.0 + 1000.0 / 3000.0;
    assert!((ground.antenna_loss.unwrap() - expected).abs() < 1e-12);
}

#[test]
fn terrain_lengthens_the_effective_distance() {
    let mut world = World::new();
    world.transmit(
        "alpha",
        TangentRadio::ShortRange,
        SW_FREQUENCY,
        3000,
        at(2500.0, 0.0, 0.0),
    );
    assert_eq!(heard(&world), [ReceivedOn::ShortRange]);
    world.client("alpha").terrain_interception = 50;
    assert_eq!(heard(&world), []);
}
