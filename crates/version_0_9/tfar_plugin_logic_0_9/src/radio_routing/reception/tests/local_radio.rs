//! Unit tests for `is_over_local_radio`, one table row per rule of `isOverLocalRadio` in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:989-1072`.

use super::*;
use crate::client_state::ClientId;
use crate::client_state::prelude::UsableRadios;

const SW: TangentKind = TangentKind::ShortRange;
const LR: TangentKind = TangentKind::LongRange;
const DD: TangentKind = TangentKind::DiverDigital;

const ALL_RADIOS: UsableRadios = UsableRadios {
    short_range: true,
    long_range: true,
    diver: true,
};

/// The listener's radios: `30.1` on short range (volume 7, left), `50` on long range (volume 9,
/// right), `both` on each (3 and 4), the diver radio on `40` at level 6, waves at 0.25 (diver
/// reach 242.5 m).
fn server() -> ServerRadioData {
    let mut server = ServerRadioData::new("Me".to_owned());
    let setting = |volume, stereo_mode| FrequencySettings {
        volume,
        stereo_mode,
    };
    server
        .sw_frequencies
        .insert("30.1".to_owned(), setting(7, StereoMode::Left));
    server
        .sw_frequencies
        .insert("both".to_owned(), setting(3, StereoMode::Both));
    server
        .lr_frequencies
        .insert("50".to_owned(), setting(9, StereoMode::Right));
    server
        .lr_frequencies
        .insert("both".to_owned(), setting(4, StereoMode::Both));
    server.dd_frequency = "40".to_owned();
    server.dd_volume_level = 6;
    server.waves_level = 0.25;
    server
}

struct Case {
    name: &'static str,
    held: Option<TangentKind>,
    frequency: &'static str,
    distance: f64,
    range: i32,
    transmitter_radios: UsableRadios,
    listener_radios: UsableRadios,
    ignore: IgnoreTangents,
    expected: Option<(TangentKind, HeardOn, i32, StereoMode)>,
}

fn case(name: &'static str, held: Option<TangentKind>, frequency: &'static str) -> Case {
    Case {
        name,
        held,
        frequency,
        distance: 100.0,
        range: 1000,
        transmitter_radios: ALL_RADIOS,
        listener_radios: ALL_RADIOS,
        ignore: IgnoreTangents::NONE,
        expected: None,
    }
}

fn run(case: &Case) -> Option<HeardVia> {
    let server = server();
    let mut transmitter = ClientData::new(ClientId(2));
    transmitter.tangent = case.held;
    transmitter.frequency = case.frequency.to_owned();
    transmitter.range = case.range;
    transmitter.usable_radios = case.transmitter_radios;
    transmitter.position = Position::from_array([case.distance, 0.0, 0.0]).unwrap();
    let mut listener = ClientData::new(ClientId(1));
    listener.usable_radios = case.listener_radios;
    is_over_local_radio(&server, &transmitter, &listener, case.ignore)
}

fn only(short_range: bool, long_range: bool, diver: bool) -> UsableRadios {
    UsableRadios {
        short_range,
        long_range,
        diver,
    }
}

fn check(cases: &[Case]) {
    for case in cases {
        let heard = run(case);
        let found = heard
            .as_ref()
            .map(|heard| (heard.over, heard.on, heard.volume, heard.stereo_mode));
        assert_eq!(found, case.expected, "{}", case.name);
    }
}

#[test]
fn every_short_and_long_range_rule_of_the_local_reception() {
    let both = StereoMode::Both;
    check(&[
        Case {
            expected: Some((SW, HeardOn::ShortRange, 7, StereoMode::Left)),
            ..case("SW to SW", Some(SW), "30.1")
        },
        Case {
            expected: Some((SW, HeardOn::LongRange, 9, StereoMode::Right)),
            ..case("SW heard on the LR list", Some(SW), "50")
        },
        Case {
            expected: Some((LR, HeardOn::ShortRange, 7, StereoMode::Left)),
            ..case("LR to SW", Some(LR), "30.1")
        },
        Case {
            expected: Some((LR, HeardOn::LongRange, 9, StereoMode::Right)),
            ..case("LR to LR", Some(LR), "50")
        },
        Case {
            expected: Some((SW, HeardOn::ShortRange, 3, both)),
            ..case(
                "SW on both lists: the SW check comes later and wins",
                Some(SW),
                "both",
            )
        },
        Case {
            expected: Some((LR, HeardOn::LongRange, 4, both)),
            ..case(
                "LR on both lists: the LR check comes later and wins",
                Some(LR),
                "both",
            )
        },
        Case {
            listener_radios: only(true, false, false),
            expected: Some((SW, HeardOn::ShortRange, 3, both)),
            ..case("SW on both lists, listener without LR", Some(SW), "both")
        },
        Case {
            listener_radios: only(false, true, false),
            expected: Some((SW, HeardOn::LongRange, 4, both)),
            ..case("SW on both lists, listener without SW", Some(SW), "both")
        },
        Case {
            distance: 1000.0,
            ..case("at exactly the range: out", Some(SW), "30.1")
        },
        Case {
            distance: 999.0,
            expected: Some((SW, HeardOn::ShortRange, 7, StereoMode::Left)),
            ..case("just inside the range", Some(SW), "30.1")
        },
        Case {
            transmitter_radios: only(false, true, true),
            ..case("transmitter cannot use SW", Some(SW), "30.1")
        },
        Case {
            listener_radios: only(false, true, true),
            ..case("listener cannot use SW", Some(SW), "30.1")
        },
        Case {
            listener_radios: only(true, false, true),
            ..case("SW on the LR list, listener cannot use LR", Some(SW), "50")
        },
        case("no tangent held", None, "30.1"),
        Case {
            ignore: IgnoreTangents::for_kind(SW),
            expected: Some((SW, HeardOn::ShortRange, 7, StereoMode::Left)),
            ..case("released, SW ignore flag", None, "30.1")
        },
        Case {
            ignore: IgnoreTangents::for_kind(DD),
            ..case(
                "released, the diver ignore flag on a short-range list",
                None,
                "30.1",
            )
        },
        case("a frequency on no list", Some(SW), "31"),
    ]);
}

#[test]
fn every_diver_radio_rule_of_the_local_reception() {
    let both = StereoMode::Both;
    check(&[
        Case {
            range: 0,
            expected: Some((DD, HeardOn::Diver, 6, both)),
            ..case(
                "DD in reach: range and terrain do not matter",
                Some(DD),
                "40",
            )
        },
        Case {
            distance: 242.0,
            expected: Some((DD, HeardOn::Diver, 6, both)),
            ..case("DD just inside the reach", Some(DD), "40")
        },
        Case {
            distance: 243.0,
            ..case("DD out of reach", Some(DD), "40")
        },
        Case {
            listener_radios: only(true, true, false),
            ..case("DD, listener without a diver radio", Some(DD), "40")
        },
        case("DD on another frequency", Some(DD), "41"),
        Case {
            ignore: IgnoreTangents::for_kind(DD),
            expected: Some((DD, HeardOn::Diver, 6, both)),
            ..case("DD released, DD ignore flag", None, "40")
        },
        case("a DD tangent never reaches the SW list", Some(DD), "30.1"),
    ]);
}

#[test]
fn terrain_and_the_multiplier_stretch_the_radio_distance() {
    let mut server = server();
    let mut transmitter = ClientData::new(ClientId(2));
    transmitter.tangent = Some(SW);
    transmitter.frequency = "30.1".to_owned();
    transmitter.range = 1000;
    transmitter.usable_radios = ALL_RADIOS;
    transmitter.position = Position::from_array([900.0, 0.0, 0.0]).unwrap();
    let mut listener = ClientData::new(ClientId(1));
    listener.usable_radios = ALL_RADIOS;
    transmitter.terrain_interception = 2;
    assert!(is_over_local_radio(&server, &transmitter, &listener, IgnoreTangents::NONE).is_some());
    transmitter.terrain_interception = 20;
    assert!(is_over_local_radio(&server, &transmitter, &listener, IgnoreTangents::NONE).is_none());
    transmitter.terrain_interception = 0;
    server.receiving_distance_multiplier = 1.2;
    assert!(is_over_local_radio(&server, &transmitter, &listener, IgnoreTangents::NONE).is_none());
}

#[test]
fn a_local_reception_comes_from_the_listeners_own_radio() {
    let server = server();
    let mut transmitter = ClientData::new(ClientId(2));
    transmitter.tangent = Some(SW);
    transmitter.frequency = "30.1".to_owned();
    transmitter.range = 1000;
    transmitter.usable_radios = ALL_RADIOS;
    let mut listener = ClientData::new(ClientId(1));
    listener.usable_radios = ALL_RADIOS;
    Arc::make_mut(&mut listener.vehicle).vehicle_name = "1:7".to_owned();
    let heard =
        is_over_local_radio(&server, &transmitter, &listener, IgnoreTangents::NONE).unwrap();
    assert_eq!(&*heard.radio_id, crate::radio_routing::LOCAL_RADIO_ID);
    assert_eq!(heard.position, Position::ORIGIN);
    assert!((heard.wave_z - 1.0).abs() < 1e-12);
    assert_eq!(heard.vehicle.vehicle_name, "1:7");
}
