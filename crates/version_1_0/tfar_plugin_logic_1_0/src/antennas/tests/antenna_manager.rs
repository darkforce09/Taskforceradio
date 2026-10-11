//! Unit tests for `AntennaManager`.

use super::*;

fn at(x: f64, y: f64) -> Position {
    Position::from_array([x, y, 0.0]).unwrap()
}

fn tower(object: i32, x: f64, y: f64, range: f64) -> Antenna {
    Antenna::new(NetId::new(2, object), at(x, y), range).unwrap()
}

#[test]
fn adding_a_known_id_replaces_the_tower_in_place() {
    let mut manager = AntennaManager::new();
    manager.add([tower(1, 0.0, 0.0, 100.0), tower(2, 5.0, 0.0, 100.0)]);
    manager.add([tower(1, 50.0, 0.0, 300.0)]);
    assert_eq!(manager.len(), 2);
    let first = manager.antennas().first().unwrap();
    assert_eq!(first.id(), NetId::new(2, 1));
    assert!((first.range() - 300.0).abs() < f64::EPSILON);
}

#[test]
fn the_null_id_is_never_added_or_removed() {
    let mut manager = AntennaManager::new();
    manager.add([Antenna::new(NetId::NULL, at(0.0, 0.0), 100.0).unwrap()]);
    assert!(manager.is_empty());
    manager.add([tower(1, 0.0, 0.0, 100.0)]);
    manager.remove([NetId::NULL, NetId::new(9, 9)]);
    assert_eq!(manager.len(), 1);
    manager.remove([NetId::new(2, 1)]);
    assert!(manager.is_empty());
}

#[test]
fn the_lowest_loss_tower_below_one_is_chosen() {
    let mut manager = AntennaManager::new();
    // Sender at 0, range 1000; receiver at 3000.
    manager.add([
        tower(1, 900.0, 0.0, 2500.0), // 900/1000 + 2100/2500 = 1.74: not below 1.
        tower(2, 200.0, 0.0, 10000.0), // 200/1000 + 2800/10000 = 0.48.
        tower(3, 100.0, 0.0, 20000.0), // 100/1000 + 2900/20000 = 0.245.
    ]);
    let connection = manager
        .find_connection(at(0.0, 0.0), 1000.0, at(3000.0, 0.0))
        .unwrap();
    assert_eq!(connection.antenna().id(), NetId::new(2, 3));
    assert!((connection.loss() - 0.245).abs() < 1e-12);
}

#[test]
fn a_tower_the_sender_cannot_reach_is_no_connection() {
    let mut manager = AntennaManager::new();
    manager.add([tower(1, 1500.0, 0.0, 20000.0)]);
    assert!(
        manager
            .find_connection(at(0.0, 0.0), 1000.0, at(3000.0, 0.0))
            .is_none()
    );
}

#[test]
fn a_tower_that_cannot_reach_the_receiver_is_no_connection() {
    let mut manager = AntennaManager::new();
    manager.add([tower(1, 100.0, 0.0, 500.0)]);
    assert!(
        manager
            .find_connection(at(0.0, 0.0), 1000.0, at(3000.0, 0.0))
            .is_none()
    );
}

#[test]
fn a_sender_without_range_connects_through_no_tower() {
    let mut manager = AntennaManager::new();
    manager.add([tower(1, 0.0, 0.0, 20000.0)]);
    assert!(
        manager
            .find_connection(at(0.0, 0.0), 0.0, at(10.0, 0.0))
            .is_none()
    );
}

#[test]
fn the_first_tower_wins_a_tie() {
    let mut manager = AntennaManager::new();
    manager.add([
        tower(1, 0.0, 100.0, 10000.0),
        tower(2, 0.0, -100.0, 10000.0),
    ]);
    let connection = manager
        .find_connection(at(0.0, 0.0), 1000.0, at(500.0, 0.0))
        .unwrap();
    assert_eq!(connection.antenna().id(), NetId::new(2, 1));
}

#[test]
fn a_tower_both_ends_reach_with_a_loss_of_one_or_more_is_no_connection() {
    let mut manager = AntennaManager::new();
    // 900/1000 + 2100/2500 = 1.74.
    manager.add([tower(1, 900.0, 0.0, 2500.0)]);
    assert!(
        manager
            .find_connection(at(0.0, 0.0), 1000.0, at(3000.0, 0.0))
            .is_none()
    );
}
