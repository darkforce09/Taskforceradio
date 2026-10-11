//! Unit tests for `rotate_about_z`.

use core::f64::consts::{FRAC_PI_2, PI};

use super::*;

fn assert_close_array(actual: [f64; 3], expected: [f64; 3]) {
    for (actual_coordinate, expected_coordinate) in actual.into_iter().zip(expected) {
        assert!(
            (actual_coordinate - expected_coordinate).abs() < 1e-12,
            "expected {expected:?}, got {actual:?}"
        );
    }
}

#[test]
fn a_quarter_turn_takes_east_to_north() {
    let east = Position::from_array([1.0, 0.0, 7.0]).unwrap();
    let turned = rotate_about_z(east, Angle::from_radians(FRAC_PI_2));
    assert_close_array(turned.to_array(), [0.0, 1.0, 7.0]);
}

#[test]
fn the_rotation_follows_the_clunk_formula() {
    let position = Position::from_array([3.0, -2.0, 1.0]).unwrap();
    let angle = Angle::from_radians(0.7);
    let (sine, cosine) = 0.7_f64.sin_cos();
    let turned = rotate_about_z(position, angle);
    assert_close_array(
        turned.to_array(),
        [3.0 * cosine + 2.0 * sine, 3.0 * sine - 2.0 * cosine, 1.0],
    );
}

#[test]
fn a_half_turn_negates_x_and_y() {
    let position = Position::from_array([2.0, 5.0, -1.0]).unwrap();
    let turned = rotate_about_z(position, Angle::from_radians(PI));
    assert_close_array(turned.to_array(), [-2.0, -5.0, -1.0]);
}

#[test]
fn rotation_keeps_the_distance_from_the_axis() {
    let position = Position::from_array([3.0, 4.0, 0.0]).unwrap();
    let turned = rotate_about_z(position, Angle::from_degrees(123.0));
    assert!((turned.distance_to(Position::ORIGIN) - 5.0).abs() < 1e-12);
}

#[test]
fn a_zero_angle_changes_nothing() {
    let position = Position::from_array([3.0, 4.0, 5.0]).unwrap();
    assert_eq!(rotate_about_z(position, Angle::ZERO), position);
}
