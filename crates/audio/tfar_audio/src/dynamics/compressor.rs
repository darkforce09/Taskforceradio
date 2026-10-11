//! The stereo-linked feed-forward compressor of the radio and direct-speech paths.
//!
//! **Role:** the SimpleComp compressor of `ChunkWare` `SimpleSource` 1.12: a peak key from the louder
//! of the two channels, its level over the threshold in decibels smoothed by an attack and
//! release envelope, and the same gain reduction applied to both channels.
//!
//! **Position:** in `crates/audio`; one instance per path in the client state crates, run by
//! the voice pipelines on each stereo block. The specification is `SimpleComp::process` in
//! `reference/tfar_1_0/ts/include/simpleSource/SimpleCompProcess.inl`, with `lin2dB` and `dB2lin`
//! from `SimpleGain.h`, and `processCompressor` in `reference/tfar_1_0/ts/src/RadioEffect.hpp`.
//!
//! **Signals and state:** each [`Compressor`] owns its over-threshold envelope; it is built off
//! the audio thread and run on the TeamSpeak audio thread, where nothing allocates.
//!
//! **Invariants:** below the threshold, with the envelope at rest, the gain is exactly 1; the gain
//! never exceeds 1 for a ratio at or below 1; one [`Compressor::process_stereo_i16`] call is one
//! pass over the block, and the caller never runs a block through it twice.

use crate::sample_buffer::StereoBuffer;

use crate::dynamics::attack_release_envelope::AttackReleaseEnvelope;
use crate::dynamics::compressor_settings::CompressorSettings;
use crate::dynamics::error::Result;

#[cfg(test)]
#[path = "tests/compressor.rs"]
mod tests;

/// The offset `SimpleSource` adds before every logarithm and to the envelope, so neither ever sees
/// zero or a denormal.
pub const DC_OFFSET: f64 = 1.0e-25;

/// `20 / ln(10)`: natural logarithm to decibels, `SimpleGain`'s `LOG_2_DB`, digit for digit.
#[allow(
    clippy::excessive_precision,
    reason = "the literal is SimpleGain's, so it rounds to the same f64 the original uses"
)]
const LOG_TO_DECIBELS: f64 = 8.685_889_638_065_036_553_022_578_378_332_1;

/// `ln(10) / 20`: decibels to natural logarithm, `SimpleGain`'s `DB_2_LOG`, digit for digit.
#[allow(
    clippy::excessive_precision,
    reason = "the literal is SimpleGain's, so it rounds to the same f64 the original uses"
)]
const DECIBELS_TO_LOG: f64 = 0.115_129_254_649_702_284_200_899_572_734_22;

/// A linear value in decibels: `ln(linear) · 20 / ln(10)`.
fn linear_to_decibels(linear: f64) -> f64 {
    linear.ln() * LOG_TO_DECIBELS
}

/// Decibels as a linear value: `exp(decibels · ln(10) / 20)`.
fn decibels_to_linear(decibels: f64) -> f64 {
    (decibels * DECIBELS_TO_LOG).exp()
}

/// A stereo-linked feed-forward compressor over samples on the raw `i16` scale.
#[derive(Debug, Clone)]
pub struct Compressor {
    settings: CompressorSettings,
    envelope: AttackReleaseEnvelope,
    over_threshold_envelope_db: f64,
}

impl Compressor {
    /// A compressor with `settings` and its envelope at rest.
    ///
    /// # Errors
    ///
    /// [`crate::dynamics::Error::InvalidSetting`] when a setting is outside its range
    /// ([`CompressorSettings::validate`]).
    pub fn new(settings: CompressorSettings) -> Result<Self> {
        settings.validate()?;
        Ok(Self {
            envelope: AttackReleaseEnvelope::new(
                settings.attack_ms,
                settings.release_ms,
                settings.sample_rate_hz,
            ),
            settings,
            over_threshold_envelope_db: DC_OFFSET,
        })
    }

    /// The settings the compressor was built with.
    pub fn settings(&self) -> &CompressorSettings {
        &self.settings
    }

    /// Puts the envelope back at rest, as SimpleComp's `initRuntime` does.
    pub fn reset(&mut self) {
        self.over_threshold_envelope_db = DC_OFFSET;
    }

    /// The smoothed level over the threshold, in decibels, after the last processed frame; 0 at
    /// rest.
    pub fn over_threshold_db(&self) -> f64 {
        self.over_threshold_envelope_db - DC_OFFSET
    }

    /// Compresses one stereo frame on the raw `i16` scale and returns it: both channels are
    /// multiplied by the same gain, keyed on the louder one.
    pub fn process_frame(&mut self, left: f64, right: f64) -> (f64, f64) {
        let key = left.abs().max(right.abs());
        let key_db = linear_to_decibels(key + DC_OFFSET);
        let over_db = (key_db - self.settings.threshold_db).max(0.0);
        self.envelope
            .run(over_db + DC_OFFSET, &mut self.over_threshold_envelope_db);
        let smoothed_over_db = self.over_threshold_envelope_db - DC_OFFSET;
        let gain = decibels_to_linear(smoothed_over_db * (self.settings.ratio - 1.0));
        (left * gain, right * gain)
    }

    /// Compresses a stereo `i16` block in place, as the originals' `processCompressor` does: each
    /// sample is used as an `f64` of its raw value and converted back by truncation toward zero,
    /// saturating at the `i16` limits. Allocates nothing.
    pub fn process_stereo_i16(&mut self, buffer: &mut StereoBuffer) {
        let (frames, _) = buffer.as_mut_slice().as_chunks_mut::<2>();
        for [left, right] in frames {
            let (compressed_left, compressed_right) =
                self.process_frame(f64::from(*left), f64::from(*right));
            *left = truncate_to_sample(compressed_left);
            *right = truncate_to_sample(compressed_right);
        }
    }
}

/// `static_cast<short>` of a compressed sample: truncation toward zero, saturating at the `i16`
/// limits (`NaN` becomes 0), so an expanding ratio never wraps.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the float to i16 `as` cast truncates toward zero and saturates, which is the intent"
)]
fn truncate_to_sample(value: f64) -> i16 {
    value as i16
}
