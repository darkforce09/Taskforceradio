//! `process_radio_effect`: one radio effect over a stereo voice block, as the 0.9.12 plugin runs
//! it (`processRadioEffect`, `reference/tfar_0_9_12/ts/src/plugin.cpp:2476`).
//!
//! **Role:** mixes the stereo block down to mono, scales it to `f32` with the gain, runs the
//! effect, and writes the result back to both channels or to one ear.
//!
//! **Position:** called by `tfar_plugin_logic_0_9::voice_pipeline` for every radio a sender is heard on, on the
//! block copied for that radio.
//!
//! **Signals and state:** none of its own; it changes the effect's state and the caller's
//! buffers. Runs on the TeamSpeak audio thread and allocates nothing once the
//! [`MonoMixBuffer`] holds the block size.
//!
//! **Invariants:** the mono sample is `trunc((left + right) / 2) / 32767 · gain`; the output
//! sample is 32767 above 1, −32768 below −1 and `trunc(sample · 32766)` between; the ear not
//! played into is silent.

#[cfg(test)]
#[path = "tests/radio_effect_mixdown.rs"]
mod tests;

use tfar_audio::sample_buffer::StereoBuffer;
use tfar_foundation::arma_values::StereoMode;

use crate::radio_effects::mono_mix_buffer::MonoMixBuffer;
use crate::radio_effects::radio_effect::RadioEffect;

/// The divisor from a mono `i16` sample to `f32`: `SHRT_MAX`.
const INPUT_SCALE: f32 = 32_767.0;

/// The multiplier from an `f32` sample inside `[-1, 1]` back to `i16`: `SHRT_MAX − 1`.
const OUTPUT_SCALE: f32 = 32_766.0;

/// The gain boost for a radio played into one ear only.
const ONE_EAR_GAIN: f32 = 1.5;

/// Runs `effect` over `buffer` in place with `gain`, routing the result by `stereo_mode`.
///
/// Each frame mixes down to `trunc((left + right) / 2)`, becomes `sample / 32767 · gain` (the
/// gain ×1.5 when the radio plays into one ear), and goes through `effect` in one block of
/// `buffer.frames()` samples held in `mono_mix`. The output goes to both channels for
/// [`StereoMode::Both`], to the left channel for [`StereoMode::Left`] and to the right for
/// [`StereoMode::Right`], the other channel silent: 32767 above 1, −32768 below −1, else
/// `trunc(sample · 32766)`.
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
    let mono = mono_mix.block(buffer.frames());
    let (frames, _) = buffer
        .as_mut_slice()
        .as_chunks_mut::<{ StereoBuffer::CHANNELS }>();
    for (&[left, right], mono_sample) in frames.iter().zip(mono.iter_mut()) {
        *mono_sample = f32::from(mono_mixdown(left, right)) / INPUT_SCALE * gain;
    }
    effect.process(mono);
    for (frame, &mono_sample) in frames.iter_mut().zip(mono.iter()) {
        let value = effect_sample_to_output(mono_sample);
        *frame = match stereo_mode {
            StereoMode::Both => [value, value],
            StereoMode::Left => [value, 0],
            StereoMode::Right => [0, value],
        };
    }
}

/// The mono sample of a frame: the sum of both channels divided by two, truncated toward zero.
fn mono_mixdown(left: i16, right: i16) -> i16 {
    i16::midpoint(left, right)
}

/// An effect output sample as `i16`: 32767 above 1, −32768 below −1, else `trunc(sample · 32766)`
/// (`NaN` gives 0).
#[expect(
    clippy::cast_possible_truncation,
    reason = "the original truncates toward zero; the value lies inside [-32766, 32766] here"
)]
fn effect_sample_to_output(sample: f32) -> i16 {
    if sample > 1.0 {
        i16::MAX
    } else if sample < -1.0 {
        i16::MIN
    } else {
        (sample * OUTPUT_SCALE) as i16
    }
}
