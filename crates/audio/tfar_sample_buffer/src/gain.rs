//! Gain on `i16` samples that saturates at full scale instead of wrapping.
//!
//! **Role:** multiplies interleaved samples by a gain, per block or per stereo channel.
//!
//! **Position:** tier 0 of crates/audio; called by the voice pipelines and the radio effects for
//! volume, distance attenuation and the serious-mode mute. The specification is
//! `reference/tfar_1_0/ts/src/SampleBuffer.cpp` (`applyMonoGain`, `applyStereoGain`) and
//! `reference/tfar_0_9_12/ts/src/plugin.cpp` (`applyGain`), with saturation in place of their
//! wrap-around.
//!
//! **Signals and state:** none; pure functions over caller buffers, called from the TeamSpeak audio
//! thread. Nothing allocates.
//!
//! **Invariants:** a scaled sample is `trunc(sample * gain)` clamped to `i16::MIN..=i16::MAX`; gain
//! 0 yields silence, gain 1 leaves the samples untouched, a `NaN` gain yields silence.

use crate::sample_saturation::saturate_to_sample;
use crate::stereo_buffer::StereoBuffer;

#[cfg(test)]
#[path = "tests/gain.rs"]
mod tests;

/// Multiplies every sample by `gain`, saturating at the `i16` limits.
///
/// Gain 0 silences the block and gain 1 returns at once, without touching a sample.
#[expect(
    clippy::float_cmp,
    reason = "the shortcuts are for the exact gains 0 and 1 the callers pass, not near values"
)]
pub fn apply_gain(samples: &mut [i16], gain: f32) {
    if gain == 0.0 {
        samples.fill(0);
        return;
    }
    if gain == 1.0 {
        return;
    }
    for sample in samples {
        *sample = scale_sample(*sample, gain);
    }
}

/// Multiplies the left channel of every frame by `left` and the right channel by `right`,
/// saturating at the `i16` limits. Gains of exactly 1 on both sides return at once.
#[expect(
    clippy::float_cmp,
    reason = "the shortcut is for the exact unity gain the callers pass, not near values"
)]
pub fn apply_stereo_gain(stereo: &mut StereoBuffer, left: f32, right: f32) {
    if left == 1.0 && right == 1.0 {
        return;
    }
    for [left_sample, right_sample] in stereo.frame_pairs_mut() {
        *left_sample = scale_sample(*left_sample, left);
        *right_sample = scale_sample(*right_sample, right);
    }
}

/// `sample * gain`, truncated toward zero and saturated to the `i16` range.
fn scale_sample(sample: i16, gain: f32) -> i16 {
    saturate_to_sample(f32::from(sample) * gain)
}
