//! Unit tests for the client filter designs, against the setups in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:232-316`.

use super::*;

#[test]
fn every_fixed_design_builds() {
    for (name, design) in [
        ("can't-speak", CANT_SPEAK_LOW_PASS),
        ("speaker", SPEAKER_BAND_PASS),
        ("phone", PHONE_BAND_PASS),
    ] {
        assert!(stereo_filter(name, design).is_ok(), "{name}");
    }
}

#[test]
fn the_vehicle_cut_off_falls_with_the_isolation() {
    let FilterDesign::ButterworthLowPass { order, cutoff_hz } = vehicle_low_pass(0.5) else {
        panic!("the vehicle filter is a Butterworth low-pass");
    };
    assert_eq!(order, 2);
    assert!((cutoff_hz - 2500.0).abs() < 1e-9);
}

#[test]
fn the_vehicle_low_pass_builds_across_the_whole_isolation_range() {
    for isolation in [0.0, 0.01, 0.5, 0.99] {
        assert!(
            stereo_filter("vehicle", vehicle_low_pass(isolation)).is_ok(),
            "{isolation}"
        );
    }
}
