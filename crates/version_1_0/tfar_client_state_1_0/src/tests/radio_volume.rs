//! Unit tests for the radio volume curve and the remote sounds.

use super::*;

#[test]
fn the_volume_curve_is_the_fourth_power_of_a_tenth_of_volume_plus_one() {
    assert!((radio_volume_gain(7.0) - 0.4096).abs() < 1e-12);
    assert!((radio_volume_gain(9.0) - 1.0).abs() < 1e-12);
    assert!((radio_volume_gain(-1.0)).abs() < 1e-12);
}

#[test]
fn only_digital_long_range_and_airborne_radios_have_remote_sounds() {
    assert_eq!(
        remote_radio_sound(&RadioSubtype::Digital, true),
        Some("radio-sounds/sw/remote_start")
    );
    assert_eq!(
        remote_radio_sound(&RadioSubtype::DigitalLongRange, false),
        Some("radio-sounds/lr/remote_end")
    );
    assert_eq!(
        remote_radio_sound(&RadioSubtype::Airborne, true),
        Some("radio-sounds/ab/remote_start")
    );
    assert_eq!(remote_radio_sound(&RadioSubtype::DiverDigital, true), None);
    assert_eq!(remote_radio_sound(&RadioSubtype::Phone, false), None);
    assert_eq!(
        remote_radio_sound(&RadioSubtype::Unknown("x".to_owned()), true),
        None
    );
}
