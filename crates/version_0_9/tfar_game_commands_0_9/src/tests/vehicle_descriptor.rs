//! Unit tests for `VehicleDescriptor`, against `getVehicleDescriptor` in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:967-988`.

use super::*;

#[test]
fn on_foot_the_whole_id_is_the_vehicle_name_with_no_isolation() {
    let descriptor = VehicleDescriptor::parse("no").unwrap();
    assert_eq!(descriptor.vehicle_name, "no");
    assert_eq!(descriptor.isolation, 0.0);
}

#[test]
fn a_turned_out_id_names_the_text_before_turnout_with_no_isolation() {
    let descriptor = VehicleDescriptor::parse("2:345_turnout").unwrap();
    assert_eq!(descriptor.vehicle_name, "2:345");
    assert_eq!(descriptor.isolation, 0.0);
}

#[test]
fn an_inside_id_splits_at_the_last_underscore_into_name_and_isolation() {
    let descriptor = VehicleDescriptor::parse("singleplayer_0.75").unwrap();
    assert_eq!(descriptor.vehicle_name, "singleplayer");
    assert_eq!(descriptor.isolation, 0.75);
    let descriptor = VehicleDescriptor::parse("a_b_0.5").unwrap();
    assert_eq!(descriptor.vehicle_name, "a_b");
    assert_eq!(descriptor.isolation, 0.5);
}

#[test]
fn a_non_numeric_isolation_is_an_error_where_stof_would_throw() {
    assert!(matches!(
        VehicleDescriptor::parse("2:345_open"),
        Err(tfar_arma_values::Error::MalformedNumber { .. })
    ));
}
