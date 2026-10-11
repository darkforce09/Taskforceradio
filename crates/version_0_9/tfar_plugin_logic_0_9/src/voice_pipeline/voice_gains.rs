//! The fixed gains of the 0.9.12 stereo function and the HRTF makeup gain.

#[cfg(test)]
#[path = "tests/voice_gains.rs"]
mod tests;

/// The gain on a muffled voice after its 100 Hz low-pass (`CANT_SPEAK_GAIN`,
/// `reference/tfar_0_9_12/ts/src/plugin.cpp:42`); also the gain of the under-water low-pass on a
/// loudspeaker.
pub const CANT_SPEAK_GAIN: f64 = 14.0;

/// The gain of a loudspeaker's band-pass (`SPEAKER_GAIN`, plugin.cpp:71).
pub const SPEAKER_GAIN: f64 = 4.0;

/// The factor on the radio volume for the short- and long-range effects (plugin.cpp:2738).
pub const RADIO_EFFECT_GAIN: f64 = 0.35;

/// The factor on the diver radio volume level for the underwater effect (plugin.cpp:2749).
pub const DIVER_RADIO_EFFECT_GAIN: f64 = 0.6;

/// The factor on the radio volume for the phone band-pass (plugin.cpp:2753).
pub const PHONE_GAIN: f64 = 10.0;

/// The exponent of the vehicle level loss: a muffled sound is scaled by `(1 − isolation)^1.2`
/// (plugin.cpp:2711).
pub const VEHICLE_LOSS_EXPONENT: f64 = 1.2;

/// The gain in decibels added after the HRTF stage so that direct speech straight ahead keeps
/// the level the original's ear-delay stage gave it.
///
/// The IRCAM sphere `tfar_audio::hrtf` renders with is diffuse-field equalised, which puts a source
/// straight ahead about 4 dB below its input (and about 9 dB below behind it). The makeup gain
/// restores the frontal level; sources to the side and behind stay quieter, as the sphere
/// measures them. Whether to keep the makeup is an open question in the
/// [audio pipeline](/documentation/architecture/audio_pipeline.md#open-questions); the ILD
/// fallback, used when no HRTF source exists, gets no makeup.
pub const HRTF_MAKEUP_GAIN_DB: f64 = 4.0;

/// [`HRTF_MAKEUP_GAIN_DB`] as a linear factor, about 1.585.
pub fn hrtf_makeup_gain() -> f64 {
    10.0_f64.powf(HRTF_MAKEUP_GAIN_DB / 20.0)
}

/// The level factor of a sound muffled by a vehicle of combined isolation `isolation`:
/// `(1 − isolation)^1.2`.
pub fn vehicle_loss_gain(isolation: f64) -> f64 {
    (1.0 - isolation).powf(VEHICLE_LOSS_EXPONENT)
}

/// `gain` as the `f32` the sample, filter and effect stages take.
#[expect(
    clippy::cast_possible_truncation,
    reason = "a gain loses only precision in f32, as the originals compute every gain in float"
)]
pub(crate) fn as_stage_gain(gain: f64) -> f32 {
    gain as f32
}
