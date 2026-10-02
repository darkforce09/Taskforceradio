//! Unit tests for `UnderWaterRadioEffect` in `tfar_radio_effects_1_0`.

use super::*;
use crate::signal_measurements::{
    ONE_SECOND, SINE_AMPLITUDE, assert_band_shape, assert_silence_stays_silent,
    band_edge_attenuations, first_sound, rms, sine, steady_state_output,
};

/// The original's band-pass: Butterworth `setup(2, 48000, 1000, 400)`.
const ORIGINAL_BAND_PASS: FilterDesign = FilterDesign::ButterworthBandPass {
    order: 2,
    centre_hz: 1000.0,
    width_hz: 400.0,
};

#[test]
fn silence_in_gives_silence_out() {
    for distance_error in [0.0, 0.5, 1.0] {
        assert_silence_stays_silent(&mut UnderWaterRadioEffect::new(1).unwrap(), distance_error);
    }
}

#[test]
fn the_same_seed_gives_the_same_output() {
    let run = |seed: u64| {
        let mut effect = UnderWaterRadioEffect::new(seed).unwrap();
        effect.set_error_level(0.5);
        let mut signal = sine(1000.0, SINE_AMPLITUDE, ONE_SECOND / 10);
        effect.process(&mut signal);
        signal
    };
    assert_eq!(run(9), run(9));
    assert_ne!(run(9), run(10));
}

#[test]
fn error_one_drops_nearly_every_sample() {
    let clean = steady_state_output(&mut UnderWaterRadioEffect::new(3).unwrap(), 0.0, 1000.0);
    let dropped = steady_state_output(&mut UnderWaterRadioEffect::new(3).unwrap(), 1.0, 1000.0);
    assert!(
        rms(&dropped) < 0.05 * rms(&clean),
        "{} against {}",
        rms(&dropped),
        rms(&clean)
    );
}

#[test]
fn the_error_level_is_the_raw_distance_error() {
    let clean = steady_state_output(&mut UnderWaterRadioEffect::new(3).unwrap(), 0.0, 1000.0);
    let half = steady_state_output(&mut UnderWaterRadioEffect::new(3).unwrap(), 0.5, 1000.0);
    let ratio = rms(&half) / rms(&clean);
    assert!(
        (0.3..0.7).contains(&ratio),
        "half the samples kept leaves {ratio}"
    );
}

#[test]
fn the_effect_has_no_delay() {
    let mut effect = UnderWaterRadioEffect::new(1).unwrap();
    let mut block = vec![0.0_f32; 480];
    block[0] = 1.0;
    effect.process(&mut block);
    assert_eq!(first_sound(&block), Some(0));
}

#[test]
fn the_band_is_centred_on_1000_hz() {
    let attenuations = band_edge_attenuations(
        || Box::new(UnderWaterRadioEffect::new(1).unwrap()),
        &[ORIGINAL_BAND_PASS],
        1000.0,
    );
    assert_band_shape(attenuations);
}
