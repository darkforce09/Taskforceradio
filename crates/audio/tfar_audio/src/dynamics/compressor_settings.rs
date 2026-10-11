//! The settings of a [`crate::dynamics::Compressor`] and the two presets of the original plugins.
//!
//! **Role:** names the threshold, ratio, attack, release and sample rate of a compressor, checks
//! their ranges, and carries the 0.9 and 1.0 presets.
//!
//! **Position:** in `crates/audio`; read by [`crate::dynamics::Compressor::new`], picked by the client
//! state crates. The presets are the `setSampleRate`, `setThresh`, `setRatio`, `setAttack` and
//! `setRelease` calls in `reference/tfar_0_9_12/ts/src/plugin.cpp` and
//! `reference/tfar_1_0/ts/src/clientData.hpp`.
//!
//! **Signals and state:** plain values; no state, nothing allocates.
//!
//! **Invariants:** the threshold is in decibels of the raw `i16` sample value, not of full scale:
//! the originals feed `static_cast<double>(short)` samples to SimpleComp, so
//! `20 · log10(32767) ≈ 90.3 dB` is full scale and the presets' 65 dB and 80 dB thresholds sit
//! about 25 dB and 10 dB below it. A validated setting is finite, and every one but the threshold
//! is above 0.

use crate::dynamics::error::{Error, Result};

#[cfg(test)]
#[path = "tests/compressor_settings.rs"]
mod tests;

/// The parameters of a feed-forward compressor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompressorSettings {
    /// The sample rate the time constants are counted at, in hertz; finite and above 0.
    pub sample_rate_hz: f64,
    /// The level above which gain reduction starts, in decibels of the raw `i16` sample value
    /// (`20 · log10(|sample|)`); finite.
    pub threshold_db: f64,
    /// The slope above the threshold: `0.1` keeps a tenth of every decibel over it (compression
    /// below 1, expansion above); finite and above 0.
    pub ratio: f64,
    /// The time constant of a rising over-threshold level, in milliseconds; finite and above 0.
    pub attack_ms: f64,
    /// The time constant of a falling over-threshold level, in milliseconds; finite and above 0.
    pub release_ms: f64,
}

impl CompressorSettings {
    /// The 0.9.12 plugin's radio compressor: 48 kHz, threshold 65 dB, ratio 0.1, attack 1 ms,
    /// release 300 ms.
    pub const VERSION_0_9: Self = Self {
        sample_rate_hz: 48_000.0,
        threshold_db: 65.0,
        ratio: 0.1,
        attack_ms: 1.0,
        release_ms: 300.0,
    };

    /// The 1.0 plugin's compressor: as [`CompressorSettings::VERSION_0_9`] with the threshold at
    /// 80 dB.
    pub const VERSION_1_0: Self = Self {
        threshold_db: 80.0,
        ..Self::VERSION_0_9
    };

    /// Checks every setting against its range.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidSetting`] naming the first setting outside its range.
    pub fn validate(&self) -> Result<()> {
        check_positive("sample_rate_hz", self.sample_rate_hz)?;
        if !self.threshold_db.is_finite() {
            return Err(Error::InvalidSetting {
                setting: "threshold_db",
                value: self.threshold_db,
                requirement: "finite",
            });
        }
        check_positive("ratio", self.ratio)?;
        check_positive("attack_ms", self.attack_ms)?;
        check_positive("release_ms", self.release_ms)
    }
}

/// Refuses a setting that is not a finite number above zero.
fn check_positive(setting: &'static str, value: f64) -> Result<()> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(Error::InvalidSetting {
            setting,
            value,
            requirement: "finite and above 0",
        })
    }
}
