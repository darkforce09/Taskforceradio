//! Unit tests for `AntennaConnection` and the no-connection loss.

use super::*;
use crate::antennas::net_id::NetId;
use tfar_foundation::geometry::Position;

#[test]
fn a_missing_connection_reads_as_the_original_loss_of_seven() {
    assert!((loss_or_no_connection(None) - 7.0).abs() < f64::EPSILON);
    assert!((loss_or_no_connection(Some(0.25)) - 0.25).abs() < f64::EPSILON);
}

#[test]
fn a_connection_keeps_its_tower_and_loss() {
    let antenna = Antenna::new(NetId::new(1, 2), Position::ORIGIN, 10.0).unwrap();
    let connection = AntennaConnection::new(antenna, 0.5);
    assert_eq!(connection.antenna().id(), NetId::new(1, 2));
    assert!((connection.loss() - 0.5).abs() < f64::EPSILON);
}
