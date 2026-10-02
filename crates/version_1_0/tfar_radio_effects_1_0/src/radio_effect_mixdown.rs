//! `process_radio_effect`: one radio effect over a stereo voice block, as the 1.0 plugin runs it
//! (`processRadioEffect`, `reference/tfar_1_0/ts/src/RadioEffect.hpp:230`).
//!
//! **Role:** mixes the stereo block down to mono, scales it to `f32` with the gain, runs the
//! effect, and writes the result back to both channels or to one ear.
//!
//! **Position:** called by `tfar_voice_pipeline_1_0` for every radio a sender is heard on and for
//! the intercom, on the block copied for that radio.
//!
//! **Signals and state:** none of its own; it changes the effect's state and the caller's
//! buffers. Runs on the TeamSpeak audio thread and allocates nothing once the
//! [`MonoMixBuffer`] holds the block size.
//!
//! **Invariants:** the mono sample is `(left + right) · (1/2 · gain · 1/32766)` in `f32`; the
//! output sample is `trunc(clamp(sample, −1, 1) · 32766)`, so it lies in `−32766..=32766`; the
//! ear not played into is silent.

#[cfg(test)]
#[path = "tests/radio_effect_mixdown.rs"]
mod tests;

use tfar_arma_values::StereoMode;
use tfar_sample_buffer::{OUTPUT_SCALE, StereoBuffer, float_to_sample};

use crate::mono_mix_buffer::MonoMixBuffer;
use crate::radio_effect::RadioEffect;

/// The gain boost for a radio played into one ear only.
const ONE_EAR_GAIN: f32 = 1.5;

/// The channels the mono mix averages: the two of a [`StereoBuffer`].
const MIXED_CHANNELS: f32 = 2.0;

/// Runs `effect` over `buffer` in place with `gain`, routing the result by `stereo_mode`.
///
/// The gain is ×1.5 when the radio plays into one ear. Each frame mixes down to
/// `(left + right) · (1/2 · gain · 1/32766)` and goes through `effect` in one block of
/// `buffer.frames()` samples held in `mono_mix`. The output, `trunc(clamp(sample, −1, 1) ·
/// 32766)`, goes to both channels for [`StereoMode::Both`], to the left channel for
/// [`StereoMode::Left`] and to the right for [`StereoMode::Right`], the other channel silent.
pub fn process_radio_effect<E: RadioEffect + ?Sized>(
    effect: &mut E,
    buffer: &mut StereoBuffer,
    gain: f32,
    stereo_mode: StereoMode,
    mono_mix: &mut MonoMixBuffer,
) {
    let gain = match stereo_mode {
        StereoMode::Both => gain,
        StereoMode::Left | StereoMode::Right => gain * ONE_EAR_GAIN,
    };
    let mono_scale = 1.0 / MIXED_CHANNELS * gain * (1.0 / OUTPUT_SCALE);
    let mono = mono_mix.block(buffer.frames());
    let (frames, _) = buffer
        .as_mut_slice()
        .as_chunks_mut::<{ StereoBuffer::CHANNELS }>();
    for (&[left, right], mono_sample) in frames.iter().zip(mono.iter_mut()) {
        *mono_sample = (f32::from(left) + f32::from(right)) * mono_scale;
    }
    effect.process(mono);
    for (frame, &mono_sample) in frames.iter_mut().zip(mono.iter()) {
        let value = float_to_sample(mono_sample);
        *frame = match stereo_mode {
            StereoMode::Both => [value, value],
            StereoMode::Left => [value, 0],
            StereoMode::Right => [0, value],
        };
    }
}
