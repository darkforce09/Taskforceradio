//! Unit tests for `Antenna`.

use super::*;

fn at(x: f64, y: f64, z: f64) -> Position {
    Position::from_array([x, y, z]).unwrap()
}

fn tower(range: f64) -> Antenna {
    Antenna::new(NetId::new(2, 1), at(0.0, 0.0, 0.0), range).unwrap()
}

#[test]
fn a_negative_or_non_finite_range_is_rejected() {
    assert!(Antenna::new(NetId::new(2, 1), at(0.0, 0.0, 0.0), -1.0).is_err());
    assert!(Antenna::new(NetId::new(2, 1), at(0.0, 0.0, 0.0), f64::NAN).is_err());
}

#[test]
fn reach_is_strictly_inside_the_range() {
    let antenna = tower(100.0);
    assert!(antenna.reaches(at(99.9, 0.0, 0.0)));
    assert!(!antenna.reaches(at(100.0, 0.0, 0.0)));
    assert!(antenna.is_reached_by(at(0.0, 49.0, 0.0), 50.0));
    assert!(!antenna.is_reached_by(at(0.0, 50.0, 0.0), 50.0));
}

#[test]
fn connection_loss_adds_both_hops_over_their_ranges() {
    let antenna = tower(1000.0);
    let loss = antenna.connection_loss(at(100.0, 0.0, 0.0), 400.0, at(0.0, 500.0, 0.0));
    assert!((loss - (500.0 / 1000.0 + 100.0 / 400.0)).abs() < 1e-12);
}

#[test]
fn a_hop_out_of_range_or_with_zero_range_costs_one() {
    let antenna = tower(0.0);
    let loss = antenna.connection_loss(at(500.0, 0.0, 0.0), 100.0, at(1.0, 0.0, 0.0));
    assert!((loss - 2.0).abs() < 1e-12);
}
