//! Unit tests for `Position`.

use glam::DVec3;

use super::*;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn from_array_keeps_the_coordinates() {
    let position = Position::from_array([1.5, -2.0, 30.25]).unwrap();
    assert_eq!(position.to_array(), [1.5, -2.0, 30.25]);
    assert_eq!(position.as_dvec3(), DVec3::new(1.5, -2.0, 30.25));
    assert_eq!(
        (position.x(), position.y(), position.z()),
        (1.5, -2.0, 30.25)
    );
}

#[test]
fn constructors_reject_non_finite_coordinates() {
    let error = Position::from_array([f64::NAN, 0.0, 0.0]).unwrap_err();
    assert!(matches!(error, Error::NonFinite { .. }));
    assert!(Position::from_array([0.0, f64::INFINITY, 0.0]).is_err());
    assert!(Position::from_dvec3(DVec3::new(0.0, 0.0, f64::NEG_INFINITY)).is_err());
}

#[test]
fn distance_to_is_the_euclidean_distance() {
    let from = Position::from_array([1.0, 2.0, 3.0]).unwrap();
    let to = Position::from_array([4.0, 6.0, 15.0]).unwrap();
    assert_close(from.distance_to(to), 13.0);
    assert_close(to.distance_to(from), 13.0);
    assert_close(from.distance_to(from), 0.0);
}

#[test]
fn direction_to_is_a_unit_vector_towards_the_other_position() {
    let from = Position::from_array([100.0, 100.0, 0.0]).unwrap();
    let to = Position::from_array([100.0, 2100.0, 0.0]).unwrap();
    let direction = from.direction_to(to);
    assert_close(direction.as_dvec3().length(), 1.0);
    assert_eq!(direction.to_array(), [0.0, 1.0, 0.0]);
}

#[test]
fn direction_to_the_same_position_is_zero() {
    let position = Position::from_array([5.0, 5.0, 5.0]).unwrap();
    assert!(position.direction_to(position).is_zero());
}

#[test]
fn the_origin_is_all_zero() {
    assert_eq!(Position::ORIGIN.to_array(), [0.0, 0.0, 0.0]);
}
