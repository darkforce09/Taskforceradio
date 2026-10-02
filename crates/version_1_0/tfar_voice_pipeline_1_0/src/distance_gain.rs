//! The 1.0 distance and vehicle gains: `volumeAttenuation` and its neighbours.
//!
//! **Role:** the level rules the 1.0 voice path applies by distance and vehicle isolation:
//! `volumeAttenuation` (`reference/tfar_1_0/ts/src/helpers.hpp:122-146`), the vehicle loss and
//! its gain, the speaker radio's audible distance and the fixed gains of `common.hpp`.
//!
//! **Position:** a module of `tfar_voice_pipeline_1_0`; read by the direct-speech and
//! radio-path planning and by the DSP needs, so the audio path and the DSP creation agree.
//!
//! **Signals and state:** none; pure functions and constants, callable from any thread.
//!
//! **Invariants:** every gain is finite and within 0 to 1 for finite input (`volume_attenuation`);
//! the vehicle loss stays within 0 to 0.99.

#[cfg(test)]
#[path = "tests/distance_gain.rs"]
mod tests;

/// The distance in metres a player who cannot speak is heard over (`CANT_SPEAK_DISTANCE`).
pub const CANT_SPEAK_DISTANCE: f64 = 5.0;

/// The gain after the 100 Hz low-pass of muffled speech (`CANT_SPEAK_GAIN`).
pub const CANT_SPEAK_GAIN: f32 = 14.0;

/// The gain after the loudspeaker band-pass (`SPEAKER_GAIN`).
pub const SPEAKER_GAIN: f32 = 4.0;

/// The wave height below which a speaker radio plays under water (`UNDERWATER_LEVEL`).
pub const UNDERWATER_LEVEL: f64 = -1.1;

/// The highest vehicle loss: some sound always gets through (`std::clamp(…, 0, 0.99)`).
pub const MAX_VEHICLE_LOSS: f64 = 0.99;

/// A vehicle loss below this counts as none: the sound is not muffled (`< 0.01`).
pub const NO_VEHICLE_LOSS: f64 = 0.01;

/// The exponent of the extra level drop through a vehicle (`pow(1 - loss, 1.2)`).
const VEHICLE_LOSS_EXPONENT: f64 = 1.2;

/// Below this the attenuation is silence (`gain < 0.001f`).
const SILENCE_GAIN: f64 = 0.001;

/// A speaker radio's volume from which on it is a placed radio heard far (`info.volume < 20`).
const LOUD_SPEAKER_VOLUME: f64 = 20.0;

/// The metres per volume step of a placed loud radio (`info.volume * 1.9f`).
const LOUD_SPEAKER_METRES_PER_VOLUME: f64 = 1.9;

/// The gain of a sound `distance` metres away whose loudest reach is `max_audible` metres times
/// `multiplier` (`volumeAttenuation`, `reference/tfar_1_0/ts/src/helpers.hpp:122-146`).
///
/// Within 1 m the gain is 1. Otherwise, with `max = max_audible · multiplier` when the players
/// can speak to each other (`should_hear`) and [`CANT_SPEAK_DISTANCE`] when they cannot, the
/// gain is `10^((distance / (2 · max)) · −60 / 20)`, which is `10^(−1.5 · distance / max)`: −30 dB
/// at `max`. A gain below 0.001 is silence and the gain never exceeds 1.
pub fn volume_attenuation(
    distance: f64,
    should_hear: bool,
    max_audible: f64,
    multiplier: f64,
) -> f64 {
    if distance <= 1.0 {
        return 1.0;
    }
    let max_distance = if should_hear {
        max_audible * multiplier
    } else {
        CANT_SPEAK_DISTANCE
    };
    let gain = 10.0_f64.powf((distance / (max_distance * 2.0)) * -60.0 / 20.0);
    if gain.is_nan() || gain < SILENCE_GAIN {
        return 0.0;
    }
    gain.min(1.0)
}

/// The isolation loss between two vehicles' sounds: the sum of both isolations, clamped to
/// `0..=0.99` (`std::clamp(myIsolation + hisIsolation, 0.0f, 0.99f)`).
pub fn vehicle_volume_loss(listener_isolation: f64, source_isolation: f64) -> f64 {
    let loss = listener_isolation + source_isolation;
    if loss.is_nan() {
        return 0.0;
    }
    loss.clamp(0.0, MAX_VEHICLE_LOSS)
}

/// The extra level factor of a sound heard through a vehicle with `loss`: `(1 − loss)^1.2`.
pub fn vehicle_loss_gain(loss: f64) -> f64 {
    (1.0 - loss).powf(VEHICLE_LOSS_EXPONENT)
}

/// How far a speaker radio at knob `volume` carries, in metres: a radio below volume 20 carries
/// `volume / 10 · speaker_distance`, a placed loud radio `volume · 1.9`
/// (`reference/tfar_1_0/ts/src/plugin.cpp:688`).
pub fn speaker_audible_distance(volume: f64, speaker_distance: f64) -> f64 {
    if volume < LOUD_SPEAKER_VOLUME {
        volume / 10.0 * speaker_distance
    } else {
        volume * LOUD_SPEAKER_METRES_PER_VOLUME
    }
}

/// `value` as the `f32` gain the sample crates take. A gain is far inside the `f32` range, so
/// the narrowing only rounds.
#[expect(
    clippy::cast_possible_truncation,
    reason = "gains are small finite values; the original computes them in f32"
)]
pub(crate) fn gain_as_f32(value: f64) -> f32 {
    value as f32
}
