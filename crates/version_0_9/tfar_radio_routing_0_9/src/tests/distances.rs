//! Unit tests for the distances, against `effectiveDistance`, `distanceForDiverRadio` and
//! `effectErrorFromDistance` in `reference/tfar_0_9_12/ts/src/plugin.cpp:932-965`.

use super::*;
use tfar_client_state_0_9::ClientId;
use tfar_client_state_0_9::prelude::Position;

fn client_at(x: f64, terrain_interception: i32) -> ClientData {
    let mut client = ClientData::new(ClientId(2));
    client.position = Position::from_array([x, 0.0, 0.0]).unwrap();
    client.terrain_interception = terrain_interception;
    client
}

fn close(left: f64, right: f64) -> bool {
    (left - right).abs() < 1e-9
}

#[test]
fn the_raw_distance_is_straight_from_the_listener() {
    let server = ServerRadioData::default();
    let mut client = client_at(3.0, 0);
    client.position = Position::from_array([3.0, 4.0, 0.0]).unwrap();
    assert!(close(raw_distance(&server, &client), 5.0));
}

#[test]
fn terrain_adds_its_coefficient_once_and_again_per_two_kilometres() {
    let server = ServerRadioData::default();
    // 1000 + 3·7 + 3·7·1000/2000
    assert!(close(
        effective_distance(&server, &client_at(1000.0, 3)),
        1031.5
    ));
    assert!(close(
        effective_distance(&server, &client_at(1000.0, 0)),
        1000.0
    ));
}

#[test]
fn the_receiving_multiplier_scales_the_whole_effective_distance() {
    let server = ServerRadioData {
        receiving_distance_multiplier: 2.0,
        terrain_intersection_coefficient: 10.0,
        ..ServerRadioData::default()
    };
    // (500 + 10 + 10·500/2000) · 2
    assert!(close(
        effective_distance(&server, &client_at(500.0, 1)),
        1025.0
    ));
}

#[test]
fn the_diver_reach_falls_from_300_to_70_metres_with_the_waves() {
    assert!(close(diver_radio_distance(0.0), DIVER_RADIO_MAX_DISTANCE));
    assert!(close(diver_radio_distance(1.0), DIVER_RADIO_MIN_DISTANCE));
    assert!(close(diver_radio_distance(0.5), 185.0));
}

#[test]
fn the_effect_error_is_the_distance_over_the_reach_of_the_family() {
    assert!(close(
        effect_error_from_distance(TangentKind::ShortRange, 500.0, 2000, 0.0),
        0.25
    ));
    assert!(close(
        effect_error_from_distance(TangentKind::LongRange, 3000.0, 2000, 0.0),
        1.5
    ));
    assert!(close(
        effect_error_from_distance(TangentKind::DiverDigital, 150.0, 9999, 0.0),
        0.5
    ));
    assert!(effect_error_from_distance(TangentKind::ShortRange, 1.0, 0, 0.0).is_infinite());
}
