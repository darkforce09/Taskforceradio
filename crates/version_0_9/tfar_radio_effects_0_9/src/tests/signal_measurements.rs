//! Test-only signal generators and measurements the effect tests share: sines, block-wise
//! processing, RMS, the share of a tone in a signal and the band shape of an effect.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "test signals are narrowed to the f32 blocks the effects take, and sample counts are far below 2^52"
)]

use tfar_dsp_filters::FilterDesign;

use crate::radio_effect::{RadioEffect, SAMPLE_RATE_HZ};

/// The block size the tests feed effects with: 10 ms, as TeamSpeak delivers.
pub(crate) const BLOCK_FRAMES: usize = 480;

/// One second of samples.
pub(crate) const ONE_SECOND: usize = 48_000;

/// The amplitude of every test sine: well inside the foldback threshold at error 0.
pub(crate) const SINE_AMPLITUDE: f64 = 0.1;

/// The least attenuation at 100 Hz and at 8 kHz, against the band centre, every effect shows.
pub(crate) const MINIMUM_EDGE_ATTENUATION_DB: f64 = 10.0;

/// How far a measured attenuation may lie from the one the filter designs predict.
pub(crate) const BAND_SHAPE_TOLERANCE_DB: f64 = 0.5;

/// `length` samples of a sine at `frequency_hz` with `amplitude`, starting at phase 0.
pub(crate) fn sine(frequency_hz: f64, amplitude: f64, length: usize) -> Vec<f32> {
    (0..length)
        .map(|index| {
            let time = index as f64 / SAMPLE_RATE_HZ;
            (amplitude * (core::f64::consts::TAU * frequency_hz * time).sin()) as f32
        })
        .collect()
}

/// Runs `effect` over `signal` in blocks of `block_frames`.
pub(crate) fn process_in_blocks(
    effect: &mut dyn RadioEffect,
    signal: &mut [f32],
    block_frames: usize,
) {
    for block in signal.chunks_mut(block_frames) {
        effect.process(block);
    }
}

/// The root mean square of `samples`.
pub(crate) fn rms(samples: &[f32]) -> f64 {
    let energy: f64 = samples
        .iter()
        .map(|sample| f64::from(*sample).powi(2))
        .sum();
    (energy / samples.len() as f64).sqrt()
}

/// The share of the energy of `samples` that lies in the single DFT bin at `frequency_hz`: 1 for
/// a pure sine with a whole number of cycles in `samples`, near 0 for noise.
pub(crate) fn tone_share(samples: &[f32], frequency_hz: f64) -> f64 {
    let (mut real, mut imaginary, mut energy) = (0.0_f64, 0.0_f64, 0.0_f64);
    for (index, sample) in samples.iter().enumerate() {
        let value = f64::from(*sample);
        let angle = core::f64::consts::TAU * frequency_hz * index as f64 / SAMPLE_RATE_HZ;
        real += value * angle.cos();
        imaginary += value * angle.sin();
        energy += value * value;
    }
    if energy == 0.0 {
        return 0.0;
    }
    2.0 * (real * real + imaginary * imaginary) / (samples.len() as f64 * energy)
}

/// One second of a test sine at `frequency_hz` through `effect` at `distance_error`, in
/// 10 ms blocks; returns the last half second.
pub(crate) fn steady_state_output(
    effect: &mut dyn RadioEffect,
    distance_error: f64,
    frequency_hz: f64,
) -> Vec<f32> {
    effect.set_error_level(distance_error);
    let mut signal = sine(frequency_hz, SINE_AMPLITUDE, ONE_SECOND);
    process_in_blocks(effect, &mut signal, BLOCK_FRAMES);
    signal.split_off(ONE_SECOND / 2)
}

/// The steady-state RMS gain of `effect` at error 0 for a sine at `frequency_hz`.
pub(crate) fn steady_state_gain(effect: &mut dyn RadioEffect, frequency_hz: f64) -> f64 {
    let output = steady_state_output(effect, 0.0, frequency_hz);
    rms(&output) / (SINE_AMPLITUDE / 2.0_f64.sqrt())
}

/// The magnitude of the cascade of `designs` at `frequency_hz`.
pub(crate) fn predicted_magnitude(designs: &[FilterDesign], frequency_hz: f64) -> f64 {
    designs
        .iter()
        .map(|design| {
            design
                .second_order_sections(SAMPLE_RATE_HZ)
                .unwrap()
                .magnitude_at(frequency_hz)
        })
        .product()
}

/// `ratio` in decibels.
pub(crate) fn decibels(ratio: f64) -> f64 {
    20.0 * ratio.log10()
}

/// The attenuation in decibels at 100 Hz and at 8 kHz against `centre_hz`, measured on effects
/// from `make_effect` and predicted from `designs`: `[(measured, predicted); 2]`.
pub(crate) fn band_edge_attenuations(
    mut make_effect: impl FnMut() -> Box<dyn RadioEffect>,
    designs: &[FilterDesign],
    centre_hz: f64,
) -> [(f64, f64); 2] {
    let centre_gain = steady_state_gain(make_effect().as_mut(), centre_hz);
    let centre_magnitude = predicted_magnitude(designs, centre_hz);
    [100.0, 8000.0].map(|edge_hz| {
        let measured = decibels(steady_state_gain(make_effect().as_mut(), edge_hz) / centre_gain);
        let predicted = decibels(predicted_magnitude(designs, edge_hz) / centre_magnitude);
        (measured, predicted)
    })
}

/// Asserts that every edge of `attenuations` lies at least [`MINIMUM_EDGE_ATTENUATION_DB`] below
/// the centre and within [`BAND_SHAPE_TOLERANCE_DB`] of the prediction.
pub(crate) fn assert_band_shape(attenuations: [(f64, f64); 2]) {
    for (measured, predicted) in attenuations {
        assert!(
            measured < -MINIMUM_EDGE_ATTENUATION_DB,
            "the edge is only {measured:.2} dB below the centre"
        );
        assert!(
            (measured - predicted).abs() < BAND_SHAPE_TOLERANCE_DB,
            "measured {measured:.2} dB, the designs predict {predicted:.2} dB"
        );
    }
}

/// Asserts that `effect` turns ten blocks of silence into silence at `distance_error`.
pub(crate) fn assert_silence_stays_silent(effect: &mut dyn RadioEffect, distance_error: f64) {
    effect.set_error_level(distance_error);
    let mut signal = vec![0.0_f32; BLOCK_FRAMES * 10];
    process_in_blocks(effect, &mut signal, BLOCK_FRAMES);
    assert!(signal.iter().all(|sample| *sample == 0.0));
}

/// The index of the first sample of `samples` that is not exactly 0.
pub(crate) fn first_sound(samples: &[f32]) -> Option<usize> {
    samples.iter().position(|sample| *sample != 0.0)
}

/// The share of its energy a tone keeps at least at error 0.
pub(crate) const CLEAN_TONE_SHARE: f64 = 0.99;

/// The share of its energy a tone keeps at most at error 1: the ring modulation and the foldback
/// move more than a tenth of it to other frequencies.
pub(crate) const DEGRADED_TONE_SHARE: f64 = 0.9;

/// Asserts that a tone at `frequency_hz` through effects from `make_effect` stays pure at error 0
/// and loses more than a tenth of its energy to other frequencies at error 1.
pub(crate) fn assert_error_one_spreads_the_tone(
    mut make_effect: impl FnMut() -> Box<dyn RadioEffect>,
    frequency_hz: f64,
) {
    let clean = steady_state_output(make_effect().as_mut(), 0.0, frequency_hz);
    let degraded = steady_state_output(make_effect().as_mut(), 1.0, frequency_hz);
    let clean_share = tone_share(&clean, frequency_hz);
    let degraded_share = tone_share(&degraded, frequency_hz);
    assert!(
        clean_share > CLEAN_TONE_SHARE,
        "error 0 keeps the tone: {clean_share}"
    );
    assert!(
        degraded_share < DEGRADED_TONE_SHARE,
        "error 1 spreads the tone: {degraded_share} against {clean_share}"
    );
}
