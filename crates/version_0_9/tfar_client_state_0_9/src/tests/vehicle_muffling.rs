//! Unit tests for `VehicleMuffling`, against `reference/tfar_0_9_12/ts/src/plugin.cpp:2691-2711`.

use super::*;

fn vehicle(id: &str) -> VehicleDescriptor {
    VehicleDescriptor::parse(id).unwrap()
}

#[test]
fn the_isolations_add_and_clamp_below_one() {
    let muffling = VehicleMuffling::between(&vehicle("1:2_0.7"), &vehicle("3:4_0.6"));
    assert!((muffling.isolation - MAX_COMBINED_ISOLATION).abs() < 1e-12);
    assert!(!muffling.is_same_vehicle);
    assert!(muffling.muffles());
    assert_eq!(muffling.muffling_isolation(), Some(MAX_COMBINED_ISOLATION));
}

#[test]
fn the_same_vehicle_never_muffles() {
    let muffling = VehicleMuffling::between(&vehicle("1:2_0.8"), &vehicle("1:2_0.8"));
    assert!(muffling.is_same_vehicle);
    assert!(!muffling.muffles());
    assert_eq!(muffling.muffling_isolation(), None);
}

#[test]
fn an_isolation_below_the_threshold_does_not_muffle() {
    let muffling = VehicleMuffling::between(&vehicle("no"), &vehicle("1:2_0.005"));
    assert!(!muffling.muffles());
    let muffling = VehicleMuffling::between(&vehicle("no"), &vehicle("1:2_0.01"));
    assert!(muffling.muffles());
}

#[test]
fn a_turned_out_crew_member_adds_no_isolation() {
    let muffling = VehicleMuffling::between(&vehicle("no"), &vehicle("1:2_turnout"));
    assert!(muffling.isolation.abs() < 1e-12);
    assert!(!muffling.muffles());
}
