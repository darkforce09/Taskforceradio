//! Unit tests for `VehicleDescriptor`, against `clientData::setVehicleId`
//! (`reference/tfar_1_0/ts/src/clientData.hpp:349-371`) and `helpers::getVehicleDescriptor`
//! (`reference/tfar_1_0/ts/src/helpers.cpp:330-350`).

use super::*;

#[test]
fn no_is_the_not_in_vehicle_descriptor() {
    // clientData.hpp:351-357.
    let descriptor = VehicleDescriptor::parse("no").unwrap();
    assert_eq!(descriptor, VehicleDescriptor::not_in_vehicle());
    assert_eq!(descriptor.name, "no");
    assert_eq!(descriptor.intercom_slot, -1);
    assert!(descriptor.isolation.abs() < f64::EPSILON);
    assert!(!descriptor.is_in_vehicle());
}

#[test]
fn three_parts_are_name_isolation_and_slot() {
    let descriptor = VehicleDescriptor::parse("2:390\x100.6\x101").unwrap();
    assert_eq!(descriptor.name, "2:390");
    assert!((descriptor.isolation - 0.6).abs() < 1e-12);
    assert_eq!(descriptor.intercom_slot, 1);
    assert_eq!(descriptor.velocity, [0.0; 3]);
    assert!(descriptor.is_in_vehicle());
}

#[test]
fn turnout_is_isolation_zero() {
    // clientData.hpp:362-363.
    let descriptor = VehicleDescriptor::parse("2:390\x10turnout\x10-1").unwrap();
    assert!(descriptor.isolation.abs() < f64::EPSILON);
    assert_eq!(descriptor.intercom_slot, -1);
}

#[test]
fn a_fourth_part_is_the_velocity() {
    // clientData.hpp:367-368; fnc_vehicleId.sqf joins `velocity _unit` as the fourth part.
    let descriptor = VehicleDescriptor::parse("2:390\x100.6\x100\x10[1.5,-2,0.25]").unwrap();
    assert_eq!(descriptor.velocity, [1.5, -2.0, 0.25]);
}

#[test]
fn a_descriptor_of_fewer_than_three_parts_is_rejected() {
    assert_eq!(VehicleDescriptor::parse("2:390\x100.6"), None);
    assert_eq!(VehicleDescriptor::parse(""), None);
}

#[test]
fn malformed_numbers_are_rejected() {
    assert_eq!(VehicleDescriptor::parse("2:390\x10thick\x101"), None);
    assert_eq!(VehicleDescriptor::parse("2:390\x100.6\x10slot"), None);
    assert_eq!(VehicleDescriptor::parse("2:390\x100.6\x1040000"), None);
    assert_eq!(
        VehicleDescriptor::parse("2:390\x100.6\x101\x10[1,x,2]"),
        None
    );
}
