//! Unit tests for `distance_underwater`.

use super::*;

fn position(x: f64, y: f64, z: f64) -> Position {
    Position::from_array([x, y, z]).unwrap()
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn both_ends_above_the_surface_give_zero() {
    assert_close(
        distance_underwater(position(0.0, 0.0, 5.0), position(30.0, 40.0, 2.0)),
        0.0,
    );
}

#[test]
fn both_ends_below_the_surface_give_the_whole_length() {
    assert_close(
        distance_underwater(position(0.0, 0.0, -1.0), position(0.0, 0.0, -11.0)),
        10.0,
    );
}

#[test]
fn a_horizontal_segment_below_the_surface_gives_the_whole_length() {
    assert_close(
        distance_underwater(position(0.0, 0.0, -3.0), position(30.0, 40.0, -3.0)),
        50.0,
    );
}

#[test]
fn a_segment_along_the_surface_gives_zero() {
    assert_close(
        distance_underwater(position(0.0, 0.0, 0.0), position(30.0, 40.0, 0.0)),
        0.0,
    );
}

#[test]
fn a_segment_from_the_surface_upwards_gives_zero() {
    assert_close(
        distance_underwater(position(0.0, 0.0, 0.0), position(0.0, 0.0, 10.0)),
        0.0,
    );
}

#[test]
fn a_segment_from_the_surface_downwards_gives_the_whole_length() {
    assert_close(
        distance_underwater(position(0.0, 0.0, 0.0), position(0.0, 0.0, -10.0)),
        10.0,
    );
}

#[test]
fn a_crossing_from_below_counts_the_submerged_part() {
    // From 2 m deep to 6 m up over 8 m east: a quarter of the 8.944 m segment is underwater.
    let from = position(0.0, 0.0, -2.0);
    let to = position(8.0, 0.0, 6.0);
    assert_close(distance_underwater(from, to), from.distance_to(to) * 0.25);
}

#[test]
fn a_crossing_from_above_counts_the_submerged_part() {
    let from = position(0.0, 0.0, 9.0);
    let to = position(0.0, 12.0, -3.0);
    assert_close(distance_underwater(from, to), from.distance_to(to) * 0.25);
}

#[test]
fn a_vertical_crossing_counts_the_depth() {
    assert_close(
        distance_underwater(position(0.0, 0.0, 4.0), position(0.0, 0.0, -1.5)),
        1.5,
    );
}

#[test]
fn the_result_is_symmetric() {
    let pairs = [
        (position(0.0, 0.0, -2.0), position(8.0, 0.0, 6.0)),
        (position(1.0, 2.0, 3.0), position(4.0, 5.0, 6.0)),
        (position(1.0, 2.0, -3.0), position(4.0, 5.0, -6.0)),
    ];
    for (from, to) in pairs {
        assert_close(distance_underwater(from, to), distance_underwater(to, from));
    }
}
