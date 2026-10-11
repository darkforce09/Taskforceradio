//! Unit tests for the fixed gains and the HRTF makeup gain.

use super::*;

#[test]
fn the_makeup_gain_is_four_decibels() {
    assert!((hrtf_makeup_gain() - 1.584_893).abs() < 1e-6);
    assert!((20.0 * hrtf_makeup_gain().log10() - HRTF_MAKEUP_GAIN_DB).abs() < 1e-12);
}

#[test]
fn the_vehicle_loss_is_the_open_share_to_the_power_one_point_two() {
    assert_eq!(vehicle_loss_gain(0.0), 1.0);
    assert!((vehicle_loss_gain(0.5) - 0.5_f64.powf(1.2)).abs() < 1e-12);
}

#[test]
fn a_stage_gain_keeps_its_value_in_f32() {
    assert_eq!(as_stage_gain(0.35), 0.35_f32);
}
