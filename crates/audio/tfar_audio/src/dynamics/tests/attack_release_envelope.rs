//! Unit tests for the attack and release envelope follower.

use super::*;

const SAMPLE_RATE_HZ: f64 = 48_000.0;

#[test]
fn the_coefficient_is_exp_of_minus_1000_over_milliseconds_times_rate() {
    let coefficient = time_constant_coefficient(1.0, SAMPLE_RATE_HZ);
    assert_eq!(coefficient, (-1.0_f64 / 48.0).exp());
    let coefficient = time_constant_coefficient(300.0, SAMPLE_RATE_HZ);
    assert_eq!(coefficient, (-1000.0_f64 / (300.0 * 48_000.0)).exp());
}

#[test]
fn a_rising_step_covers_one_minus_one_over_e_in_one_attack_time_constant() {
    let envelope = AttackReleaseEnvelope::new(1.0, 300.0, SAMPLE_RATE_HZ);
    let mut state = 0.0;
    for _ in 0..48 {
        envelope.run(1.0, &mut state);
    }
    assert!((state - (1.0 - (-1.0_f64).exp())).abs() < 1e-12, "{state}");
}

#[test]
fn a_falling_step_covers_one_minus_one_over_e_in_one_release_time_constant() {
    let envelope = AttackReleaseEnvelope::new(1.0, 300.0, SAMPLE_RATE_HZ);
    let mut state = 1.0;
    for _ in 0..14_400 {
        envelope.run(0.0, &mut state);
    }
    assert!((state - (-1.0_f64).exp()).abs() < 1e-9, "{state}");
}

#[test]
fn an_input_equal_to_the_state_leaves_it_unchanged() {
    let envelope = AttackReleaseEnvelope::new(1.0, 300.0, SAMPLE_RATE_HZ);
    let mut state = 0.25;
    envelope.run(0.25, &mut state);
    assert_eq!(state, 0.25);
}
