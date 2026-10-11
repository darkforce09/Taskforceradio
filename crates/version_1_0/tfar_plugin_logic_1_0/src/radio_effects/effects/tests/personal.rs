//! Unit tests for `PersonalRadioEffect` in `tfar_plugin_logic_1_0::radio_effects`.

use super::*;
use crate::radio_effects::signal_measurements::{
    assert_band_shape, assert_error_one_spreads_the_tone, assert_silence_stays_silent,
    band_edge_attenuations, first_sound,
};

/// The band centre: the geometric mean of the speaker corners.
const CENTRE_HZ: f64 = 1643.0;

/// The original's speaker high-pass: RBJ `setup(48000, 900, 0.85)`.
const ORIGINAL_SPEAKER_HIGH_PASS: FilterDesign = FilterDesign::RbjHighPass {
    cutoff_hz: 900.0,
    quality: 0.85,
};

/// The original's speaker low-pass: RBJ `setup(48000, 3000, 2.0)`.
const ORIGINAL_SPEAKER_LOW_PASS: FilterDesign = FilterDesign::RbjLowPass {
    cutoff_hz: 3000.0,
    quality: 2.0,
};

#[test]
fn silence_in_gives_silence_out() {
    for distance_error in [0.0, 0.5, 1.0] {
        assert_silence_stays_silent(&mut PersonalRadioEffect::new().unwrap(), distance_error);
    }
}

#[test]
fn an_impulse_first_sounds_2399_samples_late() {
    let mut effect = PersonalRadioEffect::new().unwrap();
    effect.set_error_level(0.0);
    let mut signal = vec![0.0_f32; 500 * 8];
    signal[0] = 0.5;
    for block in signal.chunks_mut(500) {
        effect.process(block);
    }
    assert_eq!(first_sound(&signal), Some(2399));
}

#[test]
fn error_one_sounds_measurably_different_from_error_zero() {
    assert_error_one_spreads_the_tone(|| Box::new(PersonalRadioEffect::new().unwrap()), 1500.0);
}

#[test]
fn the_band_runs_from_900_hz_to_3000_hz() {
    let attenuations = band_edge_attenuations(
        || Box::new(PersonalRadioEffect::new().unwrap()),
        &[ORIGINAL_SPEAKER_HIGH_PASS, ORIGINAL_SPEAKER_LOW_PASS],
        CENTRE_HZ,
    );
    assert_band_shape(attenuations);
}
