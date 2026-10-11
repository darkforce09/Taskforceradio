//! Unit tests for `ClientData`.

use std::time::Duration;

use super::*;
use crate::client_state::test_support::{ALPHA, OWN, at, now};

fn moving_client(velocity: [f64; 3], updated_at: Instant) -> ClientData {
    let mut client = ClientData::new(ALPHA, "alpha");
    client.position = at(100.0, 200.0, 10.0);
    client.position_updated_at = Some(updated_at);
    client.vehicle = Arc::new(VehicleDescriptor {
        name: "2:390".to_owned(),
        isolation: 0.5,
        intercom_slot: 1,
        velocity,
    });
    client
}

#[test]
fn a_moving_client_is_extrapolated_by_its_velocity() {
    let start = now();
    let client = moving_client([10.0, 0.0, -1.0], start);
    let later = client.client_position(start + Duration::from_secs(2));
    assert_eq!(later, at(120.0, 200.0, 8.0));
}

#[test]
fn a_client_without_velocity_stays_where_it_was_reported() {
    let start = now();
    let client = moving_client([0.0; 3], start);
    assert_eq!(
        client.client_position(start + Duration::from_secs(5)),
        at(100.0, 200.0, 10.0)
    );
}

#[test]
fn effective_distance_adds_terrain_and_scales_by_the_receiving_multiplier() {
    let start = now();
    let mut listener = ClientData::new(OWN, "me");
    listener.position = at(0.0, 0.0, 0.0);
    let mut sender = ClientData::new(ALPHA, "alpha");
    sender.position = at(1000.0, 0.0, 0.0);
    sender.terrain_interception = 10;
    let game = GameData {
        terrain_interception_coefficient: 7.0,
        receiving_distance_multiplier: 0.5,
        ..GameData::default()
    };
    // (1000 + 70 + 70 · 1000 / 2000) · 0.5
    let expected = (1000.0 + 70.0 + 35.0) * 0.5;
    let distance = listener.effective_distance_to(&sender, &game, start);
    assert!((distance - expected).abs() < 1e-9);
}

/// `is_alive` is a pure query; a stale client keeps its data frame until the sweep.
#[test]
fn is_alive_is_a_pure_query_and_never_changes_the_data_frame() {
    let start = now();
    let mut client = ClientData::new(ALPHA, "alpha");
    client.data_frame = DataFrame::new(3);
    client.last_activity_at = Some(start);
    assert!(client.is_alive(start, DataFrame::new(4)));
    assert!(!client.is_alive(start, DataFrame::new(5)));
    assert!(!client.is_alive(
        start + ALIVE_TIMEOUT + Duration::from_millis(1),
        DataFrame::new(3)
    ));
    assert_eq!(client.data_frame, DataFrame::new(3));
}

#[test]
fn a_client_is_alive_up_to_twenty_seconds_after_its_last_activity() {
    let start = now();
    let mut client = ClientData::new(ALPHA, "alpha");
    client.data_frame = DataFrame::new(3);
    assert!(!client.is_alive(start, DataFrame::new(3)));
    client.last_activity_at = Some(start);
    assert!(client.is_alive(start + ALIVE_TIMEOUT, DataFrame::new(3)));
}

#[test]
fn a_new_client_is_not_transmitting_and_in_no_vehicle() {
    let client = ClientData::new(ALPHA, "alpha");
    assert!(!client.is_transmitting());
    assert!(!client.vehicle.is_in_vehicle());
    assert!(client.can_speak);
    assert_eq!(client.nickname(), "alpha");
    assert_eq!(client.client_id(), ALPHA);
}
