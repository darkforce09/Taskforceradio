//! Unit tests for the 1.0 distance and vehicle gains.

use super::*;

#[test]
fn within_one_metre_the_attenuation_is_unity() {
    assert_eq!(volume_attenuation(0.0, true, 20.0, 1.0), 1.0);
    assert_eq!(volume_attenuation(1.0, false, 20.0, 1.0), 1.0);
}

#[test]
fn the_attenuation_is_minus_thirty_decibels_at_the_reach() {
    let gain = volume_attenuation(20.0, true, 20.0, 1.0);
    assert!((20.0 * gain.log10() - -30.0).abs() < 1e-9, "{gain}");
}

#[test]
fn the_attenuation_follows_ten_to_minus_one_and_a_half_distance_over_reach() {
    for distance in [2.0, 7.5, 13.0, 30.0] {
        let expected = 10.0_f64.powf(-1.5 * distance / 40.0);
        let gain = volume_attenuation(distance, true, 20.0, 2.0);
        assert!(
            (gain - expected).abs() < 1e-12,
            "{distance}: {gain} vs {expected}"
        );
    }
}

#[test]
fn a_player_who_cannot_speak_reaches_five_metres() {
    let gain = volume_attenuation(5.0, false, 400.0, 1.0);
    assert!((20.0 * gain.log10() - -30.0).abs() < 1e-9, "{gain}");
}

#[test]
fn below_a_thousandth_the_attenuation_is_silence() {
    assert_eq!(volume_attenuation(50.0, true, 20.0, 1.0), 0.0);
    assert!(volume_attenuation(39.0, true, 20.0, 1.0) > 0.0);
}

#[test]
fn a_zero_reach_is_silence_beyond_one_metre() {
    assert_eq!(volume_attenuation(2.0, true, 0.0, 1.0), 0.0);
}

#[test]
fn the_vehicle_loss_is_the_clamped_sum_of_both_isolations() {
    assert!((vehicle_volume_loss(0.3, 0.4) - 0.7).abs() < 1e-12);
    assert_eq!(vehicle_volume_loss(0.8, 0.8), MAX_VEHICLE_LOSS);
    assert_eq!(vehicle_volume_loss(-0.5, 0.2), 0.0);
    assert_eq!(vehicle_volume_loss(f64::NAN, 0.2), 0.0);
}

#[test]
fn the_vehicle_loss_gain_is_the_remainder_to_the_power_one_point_two() {
    assert!((vehicle_loss_gain(0.5) - 0.5_f64.powf(1.2)).abs() < 1e-12);
    assert_eq!(vehicle_loss_gain(0.0), 1.0);
}

#[test]
fn a_speaker_below_volume_twenty_carries_a_tenth_of_the_speaker_distance_per_step() {
    assert!((speaker_audible_distance(7.0, 20.0) - 14.0).abs() < 1e-12);
}

#[test]
fn a_placed_loud_speaker_carries_one_point_nine_metres_per_step() {
    assert!((speaker_audible_distance(50.0, 20.0) - 95.0).abs() < 1e-12);
}
