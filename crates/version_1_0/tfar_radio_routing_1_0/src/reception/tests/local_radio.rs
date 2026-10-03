//! Table-driven tests of the carried-radio rule.

use super::*;
use crate::fixtures::{LR_FREQUENCY, SHARED_FREQUENCY, SW_CLASSNAME, SW_FREQUENCY, World, at};
use tfar_geometry::Position;

/// One routing case: how the sender transmits, the world around, and the expected reception.
struct Case {
    name: &'static str,
    radio: TangentRadio,
    frequency: &'static str,
    range: i32,
    sender_at: Position,
    prepare: fn(&mut World),
    expected: Option<(SendingRadio, ReceivedOn)>,
}

fn nothing(_: &mut World) {}

fn heard(world: &World, ignore: IgnoreTangents) -> Option<HeardVia> {
    let context = world.context();
    let sender = world.sender("alpha");
    let antenna = world.antennas.find_connection(
        sender.client_position(world.now),
        f64::from(sender.range),
        Position::ORIGIN,
    );
    is_over_local_radio(&context, sender, ignore, antenna.as_ref())
}

const CASES: &[Case] = &[
    Case {
        name: "short range within range",
        radio: TangentRadio::ShortRange,
        frequency: SW_FREQUENCY,
        range: 3000,
        sender_at: Position::ORIGIN,
        prepare: nothing,
        expected: Some((SendingRadio::ShortRange, ReceivedOn::ShortRange)),
    },
    Case {
        name: "short range out of range without a tower",
        radio: TangentRadio::ShortRange,
        frequency: SW_FREQUENCY,
        range: 3000,
        sender_at: Position::ORIGIN,
        prepare: |world| world.client("alpha").position = at(3500.0, 0.0, 0.0),
        expected: None,
    },
    Case {
        name: "a frequency the listener does not have",
        radio: TangentRadio::ShortRange,
        frequency: "123.4",
        range: 3000,
        sender_at: Position::ORIGIN,
        prepare: nothing,
        expected: None,
    },
    Case {
        name: "long range on a long-range frequency",
        radio: TangentRadio::LongRange,
        frequency: LR_FREQUENCY,
        range: 20000,
        sender_at: Position::ORIGIN,
        prepare: nothing,
        expected: Some((SendingRadio::LongRange, ReceivedOn::LongRange)),
    },
    Case {
        name: "a frequency on both radios plays on the long-range one",
        radio: TangentRadio::ShortRange,
        frequency: SHARED_FREQUENCY,
        range: 3000,
        sender_at: Position::ORIGIN,
        prepare: nothing,
        expected: Some((SendingRadio::ShortRange, ReceivedOn::LongRange)),
    },
    Case {
        name: "without a long-range radio a shared frequency plays on the short-range one",
        radio: TangentRadio::ShortRange,
        frequency: SHARED_FREQUENCY,
        range: 3000,
        sender_at: Position::ORIGIN,
        prepare: |world| world.session.clients.own_mut().unwrap().can_use_lr_radio = false,
        expected: Some((SendingRadio::ShortRange, ReceivedOn::ShortRange)),
    },
    Case {
        name: "a long-range frequency without a long-range radio is not heard",
        radio: TangentRadio::LongRange,
        frequency: LR_FREQUENCY,
        range: 20000,
        sender_at: Position::ORIGIN,
        prepare: |world| world.session.clients.own_mut().unwrap().can_use_lr_radio = false,
        expected: None,
    },
    Case {
        name: "a diver 200 m away under water is heard as a diver radio",
        radio: TangentRadio::ShortRange,
        frequency: SW_FREQUENCY,
        range: 3000,
        sender_at: Position::ORIGIN,
        prepare: |world| {
            let alpha = world.client("alpha");
            alpha.position = at(200.0, 0.0, -5.0);
            alpha.can_use_sw_radio = false;
            alpha.can_use_dd_radio = true;
        },
        expected: Some((SendingRadio::DiverDigital, ReceivedOn::ShortRange)),
    },
    Case {
        name: "a diver 400 m away is beyond the stretched underwater range",
        radio: TangentRadio::ShortRange,
        frequency: SW_FREQUENCY,
        range: 3000,
        sender_at: Position::ORIGIN,
        prepare: |world| {
            let alpha = world.client("alpha");
            alpha.position = at(400.0, 0.0, -5.0);
            alpha.can_use_sw_radio = false;
            alpha.can_use_dd_radio = true;
        },
        expected: None,
    },
    Case {
        name: "a radio 400 m away above water is heard",
        radio: TangentRadio::ShortRange,
        frequency: SW_FREQUENCY,
        range: 3000,
        sender_at: Position::ORIGIN,
        prepare: |world| world.client("alpha").position = at(400.0, 0.0, 5.0),
        expected: Some((SendingRadio::ShortRange, ReceivedOn::ShortRange)),
    },
    Case {
        name: "half duplex blocks the radio being transmitted on",
        radio: TangentRadio::ShortRange,
        frequency: SW_FREQUENCY,
        range: 3000,
        sender_at: Position::ORIGIN,
        prepare: |world| {
            world.set_setting("full_duplex", "false");
            world.session.game.current_transmitting_radio = Some(SW_CLASSNAME.to_owned());
        },
        expected: None,
    },
    Case {
        name: "half duplex leaves another radio heard",
        radio: TangentRadio::ShortRange,
        frequency: SW_FREQUENCY,
        range: 3000,
        sender_at: Position::ORIGIN,
        prepare: |world| {
            world.set_setting("full_duplex", "false");
            world.session.game.current_transmitting_radio = Some("tf_rt1523g".to_owned());
        },
        expected: Some((SendingRadio::ShortRange, ReceivedOn::ShortRange)),
    },
    Case {
        name: "full duplex ignores the radio being transmitted on",
        radio: TangentRadio::ShortRange,
        frequency: SW_FREQUENCY,
        range: 3000,
        sender_at: Position::ORIGIN,
        prepare: |world| {
            world.session.game.current_transmitting_radio = Some(SW_CLASSNAME.to_owned());
        },
        expected: Some((SendingRadio::ShortRange, ReceivedOn::ShortRange)),
    },
    Case {
        name: "half duplex while not transmitting hears everything",
        radio: TangentRadio::ShortRange,
        frequency: SW_FREQUENCY,
        range: 3000,
        sender_at: Position::ORIGIN,
        prepare: |world| world.set_setting("full_duplex", "false"),
        expected: Some((SendingRadio::ShortRange, ReceivedOn::ShortRange)),
    },
    Case {
        name: "a tower carries a sender beyond its range",
        radio: TangentRadio::ShortRange,
        frequency: SW_FREQUENCY,
        range: 3000,
        sender_at: Position::ORIGIN,
        prepare: |world| {
            world.client("alpha").position = at(5000.0, 0.0, 0.0);
            world.add_tower(1, at(4000.0, 0.0, 0.0), 10000.0);
        },
        expected: Some((SendingRadio::ShortRange, ReceivedOn::ShortRange)),
    },
    Case {
        name: "a tower the sender cannot reach carries nothing",
        radio: TangentRadio::ShortRange,
        frequency: SW_FREQUENCY,
        range: 3000,
        sender_at: Position::ORIGIN,
        prepare: |world| {
            world.client("alpha").position = at(5000.0, 0.0, 0.0);
            world.add_tower(1, at(1500.0, 0.0, 0.0), 10000.0);
        },
        expected: None,
    },
];

#[test]
fn every_carried_radio_case_routes_as_the_original() {
    for case in CASES {
        let mut world = World::new();
        world.transmit(
            "alpha",
            case.radio,
            case.frequency,
            case.range,
            case.sender_at,
        );
        (case.prepare)(&mut world);
        let found = heard(&world, IgnoreTangents::NONE).map(|via| (via.over, via.on));
        assert_eq!(found, case.expected, "{}", case.name);
    }
}

#[test]
fn a_carried_radio_reports_its_entry_and_the_listeners_vehicle() {
    let mut world = World::new();
    world.transmit(
        "alpha",
        TangentRadio::ShortRange,
        SW_FREQUENCY,
        3000,
        Position::ORIGIN,
    );
    std::sync::Arc::make_mut(&mut world.session.clients.own_mut().unwrap().vehicle).name =
        "2:77".to_owned();
    let via = heard(&world, IgnoreTangents::NONE).unwrap();
    assert!((via.volume - 7.0).abs() < f64::EPSILON);
    assert_eq!(
        via.stereo_mode,
        tfar_client_state_1_0::prelude::StereoMode::Left
    );
    assert_eq!(&*via.radio_id, tfar_client_state_1_0::LOCAL_RADIO_ID);
    assert_eq!(via.position, None);
    assert!((via.wave_z - 1.0).abs() < f64::EPSILON);
    assert_eq!(via.vehicle.name, "2:77");
    assert_eq!(via.antenna_loss, None);
}

#[test]
fn an_antenna_loss_is_reported_even_within_direct_range() {
    let mut world = World::new();
    world.transmit(
        "alpha",
        TangentRadio::ShortRange,
        SW_FREQUENCY,
        3000,
        at(1000.0, 0.0, 0.0),
    );
    world.add_tower(1, at(500.0, 0.0, 0.0), 10000.0);
    let via = heard(&world, IgnoreTangents::NONE).unwrap();
    let expected = 500.0 / 10000.0 + 500.0 / 3000.0;
    assert!((via.antenna_loss.unwrap() - expected).abs() < 1e-12);
}

#[test]
fn the_ignore_flag_routes_a_sender_without_a_tangent() {
    let mut world = World::new();
    world.transmit(
        "alpha",
        TangentRadio::ShortRange,
        SW_FREQUENCY,
        3000,
        Position::ORIGIN,
    );
    world.client("alpha").transmitting_over = None;
    assert!(heard(&world, IgnoreTangents::NONE).is_none());
    let ignore = IgnoreTangents {
        short_range: true,
        long_range: false,
    };
    assert_eq!(
        heard(&world, ignore).map(|via| via.on),
        Some(ReceivedOn::ShortRange)
    );
}

#[test]
fn a_short_range_tangent_wins_over_a_long_range_flag() {
    let mut sender = ClientData::new(tfar_client_state_1_0::prelude::TeamspeakClientId(9), "x");
    sender.can_use_sw_radio = true;
    sender.can_use_lr_radio = true;
    let both = IgnoreTangents {
        short_range: true,
        long_range: true,
    };
    assert_eq!(
        sending_radio(&sender, both, false),
        Some(SendingRadio::ShortRange)
    );
    sender.can_use_sw_radio = false;
    assert_eq!(
        sending_radio(&sender, both, false),
        Some(SendingRadio::LongRange)
    );
    assert_eq!(sending_radio(&sender, IgnoreTangents::NONE, false), None);
}
