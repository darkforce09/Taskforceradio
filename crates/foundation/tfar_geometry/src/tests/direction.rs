//! Unit tests for `Direction`.

use core::f64::consts::{FRAC_PI_2, PI};

use glam::DVec3;

use super::*;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-12,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn from_vector_scales_to_unit_length() {
    let direction = Direction::from_vector(DVec3::new(300.0, 400.0, 0.0));
    assert_close(direction.as_dvec3().length(), 1.0);
    assert_close(direction.x(), 0.6);
    assert_close(direction.y(), 0.8);
    assert_close(direction.z(), 0.0);
}

#[test]
fn from_vector_gives_unit_length_for_every_scale() {
    for scale in [1e-6, 0.5, 1.0, 2.0, 100.0, 25_000.0, 1e9] {
        let direction = Direction::from_vector(DVec3::new(1.0, -2.0, 3.0) * scale);
        assert_close(direction.as_dvec3().length(), 1.0);
    }
}

#[test]
fn from_vector_keeps_zero_as_zero() {
    let direction = Direction::from_vector(DVec3::ZERO);
    assert_eq!(direction, Direction::ZERO);
    assert!(direction.is_zero());
}

#[test]
fn from_vector_turns_a_non_finite_vector_into_zero() {
    assert!(Direction::from_vector(DVec3::new(f64::NAN, 0.0, 1.0)).is_zero());
    assert!(Direction::from_vector(DVec3::new(f64::INFINITY, 0.0, 0.0)).is_zero());
}

#[test]
fn from_array_normalises_finite_coordinates() {
    let direction = Direction::from_array([0.0, 0.0, -7.0]).unwrap();
    assert_eq!(direction.to_array(), [0.0, 0.0, -1.0]);
    assert!(!direction.is_zero());
}

#[test]
fn from_array_rejects_non_finite_coordinates() {
    let error = Direction::from_array([1.0, f64::NAN, 0.0]).unwrap_err();
    assert!(matches!(error, Error::NonFinite { .. }));
    assert!(Direction::from_array([f64::NEG_INFINITY, 0.0, 0.0]).is_err());
}

#[test]
fn view_angle_is_the_compass_heading() {
    let north = Direction::from_array([0.0, 1.0, 0.0]).unwrap();
    let east = Direction::from_array([1.0, 0.0, 0.0]).unwrap();
    let west = Direction::from_array([-1.0, 0.0, 0.0]).unwrap();
    let south = Direction::from_array([0.0, -1.0, 0.0]).unwrap();
    assert_close(north.view_angle().radians(), 0.0);
    assert_close(east.view_angle().radians(), FRAC_PI_2);
    assert_close(west.view_angle().radians(), -FRAC_PI_2);
    assert_close(south.view_angle().radians(), PI);
}

#[test]
fn polar_angle_is_measured_from_east_towards_north() {
    let north = Direction::from_array([0.0, 1.0, 0.0]).unwrap();
    let east = Direction::from_array([1.0, 0.0, 0.0]).unwrap();
    let north_east = Direction::from_array([1.0, 1.0, 5.0]).unwrap();
    assert_close(east.polar_angle().radians(), 0.0);
    assert_close(north.polar_angle().radians(), FRAC_PI_2);
    assert_close(north_east.polar_angle().to_degrees(), 45.0);
}

#[test]
fn the_zero_direction_has_angle_zero() {
    assert_close(Direction::ZERO.view_angle().radians(), 0.0);
    assert_close(Direction::ZERO.polar_angle().radians(), 0.0);
}
