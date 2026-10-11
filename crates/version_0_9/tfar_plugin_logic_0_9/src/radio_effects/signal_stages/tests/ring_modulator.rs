//! Unit tests for `RingModulator` in `tfar_plugin_logic_0_9::radio_effects`.

#![allow(
    clippy::cast_precision_loss,
    reason = "the test steps are small integers, exact as f32"
)]

use super::*;

#[test]
fn a_mix_of_zero_passes_the_input_unchanged() {
    let mut modulator = RingModulator::default();
    for step in 0..2000 {
        let input = (step as f32 * 0.01).sin();
        assert_eq!(modulator.modulate(input, 0.0), input);
    }
}

#[test]
fn the_carrier_ramps_a_quarter_sine_and_starts_over_after_534_samples() {
    let mut modulator = RingModulator::default();
    let carrier: Vec<f32> = (0..1100).map(|_| modulator.modulate(1.0, 1.0)).collect();
    assert_eq!(carrier[0], 0.0);
    assert!(carrier[533] > 0.999, "{}", carrier[533]);
    assert_eq!(carrier[534], 0.0);
    assert_eq!(carrier[1068], 0.0);
    assert!(carrier.windows(2).take(533).all(|pair| pair[1] > pair[0]));
}
