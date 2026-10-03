//! Table-driven tests of `is_over_radio`: intercom, range, speakers and towers.

use std::sync::Arc;

use super::*;
use crate::fixtures::{ALPHA, BRAVO, SW_FREQUENCY, World, at};
use tfar_client_state_1_0::prelude::{
    ReceivedOn, SendingRadio, Speaker, StereoMode, TangentRadio, VehicleDescriptor,
};
use tfar_geometry::Position;

/// A reception of an earlier frame, left in a reused list.
fn stale_reception() -> HeardVia {
    HeardVia {
        over: SendingRadio::DiverDigital,
        on: ReceivedOn::LongRange,
        volume: -1.0,
        stereo_mode: StereoMode::Right,
        radio_id: "stale".into(),
        position: None,
        wave_z: 0.0,
        vehicle: Arc::new(VehicleDescriptor::not_in_vehicle()),
        antenna_loss: None,
    }
}

/// Where the local player hears `alpha`; every case also checks that `is_over_radio_into`, into
/// a list holding a stale reception, gives the same list as `is_over_radio`.
fn heard(world: &World) -> Vec<ReceivedOn> {
    let context = world.context();
    let sender = world.sender("alpha");
    let heard = is_over_radio(&context, sender, IgnoreTangents::NONE);
    let mut reused = vec![stale_reception()];
    is_over_radio_into(&context, sender, IgnoreTangents::NONE, &mut reused);
    assert_eq!(reused, heard);
    heard.into_iter().map(|via| via.on).collect()
}

fn in_vehicle(world: &mut World, nickname: &str, slot: i16) {
    world.client(nickname).vehicle = Arc::new(VehicleDescriptor {
        name: "2:390".to_owned(),
        isolation: 0.5,
        intercom_slot: slot,
        velocity: [0.0; 3],
    });
}

fn speaker(
    owner: Option<tfar_client_state_1_0::prelude::TeamspeakClientId>,
    position: Option<Position>,
) -> Speaker {
    Speaker {
        radio_id: "2:500".into(),
        owner,
        position,
        volume: 8.0,
        vehicle: Arc::new(VehicleDescriptor::not_in_vehicle()),
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
    assert_eq!(&*via.first().unwrap().radio_id, "intercom");
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
                Arc::make_mut(&mut world.session.clients.own_mut().unwrap().vehicle)
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
    assert_eq!(&*ground.radio_id, "2:500");
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

/// `alpha` transmits on short range 100 m away, heard on a carried radio and on two speakers.
fn world_with_speakers() -> World {
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
        vec![
            speaker(Some(BRAVO), Some(at(5.0, 5.0, 0.0))),
            speaker(Some(BRAVO), Some(at(6.0, 5.0, 0.0))),
        ],
    );
    world
}

#[test]
fn a_reception_shares_its_radio_id_and_vehicle_with_the_session() {
    let world = world_with_speakers();
    let via = is_over_radio(
        &world.context(),
        world.sender("alpha"),
        IgnoreTangents::NONE,
    );
    let own = world.session.clients.own().unwrap();
    let speaker = &world.session.game.speakers_on(SW_FREQUENCY)[0];
    assert_eq!(via.len(), 3);
    assert!(Arc::ptr_eq(
        &via[0].radio_id,
        &tfar_client_state_1_0::local_radio_id()
    ));
    assert!(Arc::ptr_eq(&via[0].vehicle, &own.vehicle));
    assert!(Arc::ptr_eq(&via[1].radio_id, &speaker.radio_id));
    assert!(Arc::ptr_eq(&via[1].vehicle, &speaker.vehicle));
}

#[test]
fn filling_a_list_with_nothing_heard_drops_what_it_held() {
    let world = World::new();
    let mut reused = vec![stale_reception(), stale_reception()];
    is_over_radio_into(
        &world.context(),
        world.sender("alpha"),
        IgnoreTangents::NONE,
        &mut reused,
    );
    assert_eq!(reused, []);
}

#[test]
fn a_thousand_frames_into_a_warm_list_never_reallocate_it() {
    let world = world_with_speakers();
    let context = world.context();
    let (alpha, bravo) = (world.sender("alpha"), world.sender("bravo"));
    let mut reused = Vec::new();
    is_over_radio_into(&context, alpha, IgnoreTangents::NONE, &mut reused);
    let (capacity, buffer) = (reused.capacity(), reused.as_ptr());
    for frame in 0..1_000 {
        let (sender, expected) = if frame % 3 == 0 {
            (bravo, 0)
        } else {
            (alpha, 3)
        };
        is_over_radio_into(&context, sender, IgnoreTangents::NONE, &mut reused);
        assert_eq!(reused.len(), expected);
        assert_eq!((reused.capacity(), reused.as_ptr()), (capacity, buffer));
    }
}
