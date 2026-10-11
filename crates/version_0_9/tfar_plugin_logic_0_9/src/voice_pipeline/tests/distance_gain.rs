//! Unit tests for `volume_from_distance` and `radio_volume_multiplier`, against
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:698` and `:2569`.

use super::*;

#[test]
fn within_one_metre_the_voice_keeps_its_full_level() {
    assert_eq!(volume_from_distance(0.0, true, 20, 1.0), 1.0);
    assert_eq!(volume_from_distance(1.0, false, 0, 1.0), 1.0);
}

#[test]
fn beyond_one_metre_the_gain_follows_the_power_and_linear_fall_off() {
    let gain = volume_from_distance(10.0, true, 20, 1.0);
    let expected = 10.0_f64.powf(-0.3) * (10.0 / 20.0);
    assert!((gain - expected).abs() < 1e-12, "{gain} vs {expected}");
}

#[test]
fn the_multiplier_scales_the_reach() {
    let gain = volume_from_distance(10.0, true, 20, 0.6);
    let expected = 10.0_f64.powf(-0.3) * (12.0 - 10.0) / 12.0;
    assert!((gain - expected).abs() < 1e-12);
}

#[test]
fn a_voice_that_cannot_be_heard_carries_five_metres_whatever_its_volume() {
    let at_four = volume_from_distance(4.0, false, 1000, 1.0);
    let expected = 4.0_f64.powf(-0.3) * (5.0 - 4.0) / 5.0;
    assert!((at_four - expected).abs() < 1e-12);
    assert_eq!(volume_from_distance(6.0, false, 1000, 1.0), 0.0);
}

#[test]
fn beyond_the_reach_and_below_the_audible_floor_the_gain_is_zero() {
    assert_eq!(volume_from_distance(25.0, true, 20, 1.0), 0.0);
    assert_eq!(volume_from_distance(19.999, true, 20, 1.0), 0.0);
}

#[test]
fn a_zero_reach_is_silence_not_a_division_by_zero() {
    assert_eq!(volume_from_distance(3.0, true, 0, 1.0), 0.0);
    assert_eq!(volume_from_distance(3.0, true, 20, 0.0), 0.0);
}

#[test]
fn the_gain_never_exceeds_one() {
    for tenth in 11..200 {
        let gain = volume_from_distance(f64::from(tenth) / 10.0, true, 30, 1.0);
        assert!((0.0..=1.0).contains(&gain));
    }
}

#[test]
fn the_radio_volume_curve_is_the_fourth_power_of_a_tenth_past_the_volume() {
    assert!((radio_volume_multiplier(9) - 1.0).abs() < 1e-12);
    assert!((radio_volume_multiplier(7) - 0.4096).abs() < 1e-12);
    assert_eq!(radio_volume_multiplier(-1), 0.0);
}
