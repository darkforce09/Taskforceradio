//! Unit tests for `ListenerRelativeDirection`: Arma world axes and compass headings into the
//! listener's frame, and the listener's frame into the sphere's sampling vector.

use super::*;

const TOLERANCE: f64 = 1e-12;

fn position(x: f64, y: f64, z: f64) -> Position {
    Position::from_array([x, y, z]).unwrap()
}

fn assert_axes(direction: ListenerRelativeDirection, right: f64, forward: f64, up: f64) {
    assert!(
        (direction.right() - right).abs() < TOLERANCE
            && (direction.forward() - forward).abs() < TOLERANCE
            && (direction.up() - up).abs() < TOLERANCE,
        "{direction:?}, expected right {right}, forward {forward}, up {up}"
    );
}

#[test]
fn facing_north_a_source_to_the_north_is_ahead_and_one_to_the_east_is_right() {
    let listener = position(1000.0, 2000.0, 5.0);
    let north = Angle::ZERO;
    let ahead =
        ListenerRelativeDirection::from_arma(listener, north, position(1000.0, 2030.0, 5.0));
    assert_axes(ahead, 0.0, 1.0, 0.0);
    let east = ListenerRelativeDirection::from_arma(listener, north, position(1010.0, 2000.0, 5.0));
    assert_axes(east, 1.0, 0.0, 0.0);
    let west = ListenerRelativeDirection::from_arma(listener, north, position(990.0, 2000.0, 5.0));
    assert_axes(west, -1.0, 0.0, 0.0);
    let south =
        ListenerRelativeDirection::from_arma(listener, north, position(1000.0, 1900.0, 5.0));
    assert_axes(south, 0.0, -1.0, 0.0);
}

#[test]
fn facing_east_a_source_to_the_south_is_right_and_one_to_the_north_is_left() {
    let listener = position(0.0, 0.0, 0.0);
    let east = Angle::from_degrees(90.0);
    let south = ListenerRelativeDirection::from_arma(listener, east, position(0.0, -5.0, 0.0));
    assert_axes(south, 1.0, 0.0, 0.0);
    let north = ListenerRelativeDirection::from_arma(listener, east, position(0.0, 5.0, 0.0));
    assert_axes(north, -1.0, 0.0, 0.0);
    let ahead = ListenerRelativeDirection::from_arma(listener, east, position(7.0, 0.0, 0.0));
    assert_axes(ahead, 0.0, 1.0, 0.0);
}

#[test]
fn facing_south_west_a_source_to_the_north_west_is_right() {
    let listener = position(0.0, 0.0, 0.0);
    let south_west = Angle::from_degrees(225.0);
    let north_west =
        ListenerRelativeDirection::from_arma(listener, south_west, position(-3.0, 3.0, 0.0));
    assert_axes(north_west, 1.0, 0.0, 0.0);
}

#[test]
fn height_stays_up_whatever_the_heading() {
    let listener = position(0.0, 0.0, 10.0);
    for degrees in [0.0, 37.0, 180.0, 300.0] {
        let above = ListenerRelativeDirection::from_arma(
            listener,
            Angle::from_degrees(degrees),
            position(0.0, 0.0, 30.0),
        );
        assert_axes(above, 0.0, 0.0, 1.0);
    }
}

#[test]
fn the_direction_is_scaled_to_unit_length() {
    let direction = ListenerRelativeDirection::from_listener_axes(3.0, 4.0, 0.0);
    assert_axes(direction, 0.6, 0.8, 0.0);
}

#[test]
fn a_source_at_the_listener_or_a_non_finite_vector_is_straight_ahead() {
    let here = position(5.0, 5.0, 5.0);
    assert_eq!(
        ListenerRelativeDirection::from_arma(here, Angle::from_degrees(45.0), here),
        ListenerRelativeDirection::STRAIGHT_AHEAD
    );
    assert_eq!(
        ListenerRelativeDirection::from_listener_axes(f64::NAN, 1.0, 0.0),
        ListenerRelativeDirection::STRAIGHT_AHEAD
    );
    assert_eq!(
        ListenerRelativeDirection::from_listener_axes(0.0, 0.0, 0.0),
        ListenerRelativeDirection::STRAIGHT_AHEAD
    );
}

#[test]
fn the_sampling_vector_is_x_right_y_up_z_backwards() {
    let vector = |right, forward, up| {
        let sampled = ListenerRelativeDirection::from_listener_axes(right, forward, up)
            .sphere_sampling_vector();
        [sampled.x, sampled.y, sampled.z]
    };
    assert_eq!(vector(0.0, 1.0, 0.0), [0.0, 0.0, -1.0]);
    assert_eq!(vector(0.0, -1.0, 0.0), [0.0, 0.0, 1.0]);
    assert_eq!(vector(1.0, 0.0, 0.0), [1.0, 0.0, 0.0]);
    assert_eq!(vector(-1.0, 0.0, 0.0), [-1.0, 0.0, 0.0]);
    assert_eq!(vector(0.0, 0.0, 1.0), [0.0, 1.0, 0.0]);
}
