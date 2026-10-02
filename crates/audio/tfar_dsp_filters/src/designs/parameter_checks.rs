//! The checks every filter design runs on its parameters before it calls a design library.
//!
//! The `sci-rs` design functions panic on a frequency outside `(0, Nyquist)` and on band edges in
//! the wrong order, and this crate runs with `panic = "abort"`, so each check here is strict
//! enough that no design call can reach those panics. The frequency check uses the same
//! expression as `sci-rs` (`2 * f / fs`), so a value accepted here is accepted there.

use crate::error::{Error, Result};
use crate::filter_design::MAX_BUTTERWORTH_ORDER;

#[cfg(test)]
#[path = "tests/parameter_checks.rs"]
mod tests;

/// Refuses a sample rate that is not a finite number above zero.
pub(crate) fn check_sample_rate(sample_rate_hz: f64) -> Result<()> {
    if sample_rate_hz.is_finite() && sample_rate_hz > 0.0 {
        Ok(())
    } else {
        Err(Error::InvalidSampleRate { sample_rate_hz })
    }
}

/// Refuses a design frequency that is not strictly between 0 and Nyquist, in the normalised form
/// `2 * frequency / sample_rate` the design libraries compute. Returns that normalised value.
pub(crate) fn check_design_frequency(
    frequency_role: &'static str,
    frequency_hz: f64,
    sample_rate_hz: f64,
) -> Result<f64> {
    let normalised = 2.0 * frequency_hz / sample_rate_hz;
    if frequency_hz.is_finite() && normalised > 0.0 && normalised < 1.0 {
        Ok(normalised)
    } else {
        Err(Error::FrequencyOutOfRange {
            frequency_role,
            frequency_hz,
            nyquist_hz: sample_rate_hz / 2.0,
        })
    }
}

/// Refuses an RBJ quality factor that is not a finite number above zero.
pub(crate) fn check_quality(quality: f64) -> Result<()> {
    if quality.is_finite() && quality > 0.0 {
        Ok(())
    } else {
        Err(Error::InvalidQuality { quality })
    }
}

/// Refuses a Butterworth order outside `1..=MAX_BUTTERWORTH_ORDER`.
pub(crate) fn check_order(order: usize) -> Result<()> {
    if (1..=MAX_BUTTERWORTH_ORDER).contains(&order) {
        Ok(())
    } else {
        Err(Error::InvalidOrder {
            order,
            maximum: MAX_BUTTERWORTH_ORDER,
        })
    }
}
