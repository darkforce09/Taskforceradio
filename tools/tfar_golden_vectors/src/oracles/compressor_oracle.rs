//! The SimpleComp oracle: the original `ChunkWare` `simpleSource` compressor, run over test signals.
//!
//! **Role:** checks compressor settings and runs SimpleComp through the shim on interleaved
//! stereo `f64` frames, returning the compressed frames and the gain reduction of every frame, and
//! on stereo `i16` blocks as the original `processCompressor` does.
//!
//! **Position:** inside `oracles`; called by the compressor comparison tests, which run the same
//! signal through `tfar_dynamics`. The specification is `SimpleComp` in
//! `reference/tfar_1_0/ts/include/simpleSource/`, set up in the order of
//! `reference/tfar_1_0/ts/src/clientData.hpp` (rate, threshold, release, attack, ratio,
//! `initRuntime`).
//!
//! **Signals and state:** none kept: every call sets up a fresh compressor with its envelope at
//! rest and drops it. Any thread may call.
//!
//! **Invariants:** the gain of a frame is the factor SimpleComp multiplies both channels by,
//! keyed on `max(|left|, |right|)`; samples are on the raw `i16` scale the originals compress on.

use crate::error::{Error, Result};
use crate::oracles::oracle_bindings::{self, ShimCompressorSettings};
use crate::oracles::parameter_checks::check_positive;

#[cfg(test)]
#[path = "tests/compressor_oracle.rs"]
mod tests;

/// The parameters of one SimpleComp, as its setters take them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OracleCompressorSettings {
    /// `setSampleRate`, in hertz.
    pub sample_rate_hz: f64,
    /// `setThresh`, in decibels of the raw `i16` sample value.
    pub threshold_db: f64,
    /// `setRatio`: below 1 compresses, above 1 expands.
    pub ratio: f64,
    /// `setAttack`, in milliseconds.
    pub attack_ms: f64,
    /// `setRelease`, in milliseconds.
    pub release_ms: f64,
}

/// What SimpleComp made of a signal: the compressed frames and each frame's linear gain.
#[derive(Debug, Clone, PartialEq)]
pub struct OracleCompressorRun {
    /// The compressed interleaved stereo frames.
    pub frames: Vec<[f64; 2]>,
    /// The linear gain SimpleComp applied to each frame.
    pub gains: Vec<f64>,
}

/// A checked SimpleComp configuration, ready to run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OracleCompressor {
    shim_settings: ShimCompressorSettings,
}

impl OracleCompressor {
    /// Checks `settings`.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidParameter`] for a threshold that is not finite, or another setting that
    /// is not finite and above 0 (SimpleComp asserts a positive ratio).
    pub fn new(settings: OracleCompressorSettings) -> Result<Self> {
        check_positive("sample_rate_hz", settings.sample_rate_hz)?;
        if !settings.threshold_db.is_finite() {
            return Err(Error::InvalidParameter {
                parameter: "threshold_db",
                value: settings.threshold_db,
                requirement: "finite",
            });
        }
        check_positive("ratio", settings.ratio)?;
        check_positive("attack_ms", settings.attack_ms)?;
        check_positive("release_ms", settings.release_ms)?;
        Ok(Self {
            shim_settings: ShimCompressorSettings {
                sample_rate_hz: settings.sample_rate_hz,
                threshold_db: settings.threshold_db,
                ratio: settings.ratio,
                attack_ms: settings.attack_ms,
                release_ms: settings.release_ms,
            },
        })
    }

    /// Runs `frames` through `SimpleComp::process(double&, double&)` from rest.
    ///
    /// # Errors
    ///
    /// [`Error::OracleRefused`] when the shim refuses the call.
    pub fn process_frames(&self, frames: &[[f64; 2]]) -> Result<OracleCompressorRun> {
        let mut compressed = frames.to_vec();
        let mut gains = vec![0.0; frames.len()];
        oracle_bindings::simple_comp_process(
            &self.shim_settings,
            compressed.as_flattened_mut(),
            &mut gains,
        )?;
        Ok(OracleCompressorRun {
            frames: compressed,
            gains,
        })
    }

    /// Compresses interleaved stereo `i16` samples in place as `processCompressor`
    /// (`reference/tfar_1_0/ts/src/RadioEffect.hpp`) does, from rest.
    ///
    /// # Errors
    ///
    /// [`Error::UnevenFrames`] for an odd number of samples; [`Error::OracleRefused`] when the
    /// shim refuses the call.
    pub fn process_stereo_i16(&self, samples: &mut [i16]) -> Result<()> {
        oracle_bindings::simple_comp_process_stereo_i16(&self.shim_settings, samples)
    }
}
