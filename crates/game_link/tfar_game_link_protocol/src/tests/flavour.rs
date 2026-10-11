//! Unit tests for `Flavour`.

use super::*;

#[test]
fn the_flavour_bytes_are_nine_for_0_9_and_ten_for_1_0() {
    assert_eq!(Flavour::V0_9.wire_value(), 9);
    assert_eq!(Flavour::V1_0.wire_value(), 10);
}

#[test]
fn every_flavour_byte_maps_back_to_its_flavour_and_others_to_none() {
    for flavour in [Flavour::V0_9, Flavour::V1_0] {
        assert_eq!(
            Flavour::from_wire_value(flavour.wire_value()),
            Some(flavour)
        );
    }
    for value in (0..=u8::MAX).filter(|value| *value != 9 && *value != 10) {
        assert_eq!(Flavour::from_wire_value(value), None);
    }
}
