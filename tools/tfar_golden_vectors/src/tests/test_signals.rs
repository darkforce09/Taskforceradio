//! Unit tests for `test_signals`.

use super::*;

#[test]
fn the_same_seed_gives_the_same_noise_and_another_seed_different_noise() {
    assert_eq!(white_noise(256, 1.0, 7), white_noise(256, 1.0, 7));
    assert_ne!(white_noise(256, 1.0, 7), white_noise(256, 1.0, 8));
}

#[test]
fn every_signal_has_its_length_and_stays_within_its_amplitude() {
    let signals = [
        white_noise(1_000, 0.5, 1),
        sine_sweep(1_000, 20.0, 20_000.0, 0.5, SAMPLE_RATE_HZ),
        impulse(1_000, 0.5),
        modulated_noise_burst(1_000, 0.5, 4.0, SAMPLE_RATE_HZ, 1),
        level_steps(&[0.5, 0.25], 500),
    ];
    for signal in signals {
        assert_eq!(signal.len(), 1_000);
        assert!(
            signal
                .iter()
                .all(|sample| sample.is_finite() && sample.abs() <= 0.5)
        );
    }
}

#[test]
fn the_noise_is_roughly_uniform_with_zero_mean() {
    let noise = white_noise(100_000, 1.0, 3);
    let mean = noise.iter().sum::<f64>() / 100_000.0;
    let mean_square = noise.iter().map(|sample| sample * sample).sum::<f64>() / 100_000.0;
    assert!(mean.abs() < 0.01);
    assert!((mean_square - 1.0 / 3.0).abs() < 0.01);
}

#[test]
fn an_impulse_is_one_sample_high() {
    let signal = impulse(4, 2.0);
    assert_eq!(signal, [2.0, 0.0, 0.0, 0.0]);
}

#[test]
fn level_steps_alternate_their_sign_every_sample() {
    assert_eq!(level_steps(&[3.0, 1.0], 2), [3.0, -3.0, 1.0, -1.0]);
}

#[test]
fn log_spaced_frequencies_include_both_ends() {
    let frequencies = log_spaced_frequencies(3, 10.0, 1_000.0);
    assert!((frequencies[0] - 10.0).abs() < 1.0e-9);
    assert!((frequencies[1] - 100.0).abs() < 1.0e-9);
    assert!((frequencies[2] - 1_000.0).abs() < 1.0e-9);
}

#[test]
fn the_burst_is_twenty_times_louder_in_its_middle_half() {
    let signal = modulated_noise_burst(48_000, 1.0, 4.0, SAMPLE_RATE_HZ, 5);
    let peak = |range: core::ops::Range<usize>| {
        signal[range]
            .iter()
            .fold(0.0_f64, |peak, sample| peak.max(sample.abs()))
    };
    assert!(peak(12_000..36_000) > 0.9);
    assert!(peak(0..12_000) <= 0.05);
}
