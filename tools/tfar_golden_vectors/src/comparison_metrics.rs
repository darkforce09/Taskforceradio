//! The error measures the comparisons judge the Rust crates by against the oracle.
//!
//! **Role:** root mean square, RMS error relative to the reference, the largest absolute
//! difference, and the decibel distance between two linear gains.
//!
//! **Position:** used by the comparison tests of this crate; nothing else calls it.
//!
//! **Signals and state:** none; pure functions.
//!
//! **Invariants:** sequences are compared sample by sample and must have the same length, or the
//! measure is an [`Error::LengthMismatch`]; the reference is always the oracle's output.

use crate::error::{Error, Result};

#[cfg(test)]
#[path = "tests/comparison_metrics.rs"]
mod tests;

/// The root mean square of `values`; 0 for an empty slice.
pub fn root_mean_square(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum_of_squares: f64 = values.iter().map(|value| value * value).sum();
    (sum_of_squares / length_to_f64(values.len())).sqrt()
}

/// The RMS of `candidate − reference` divided by the RMS of `reference`.
///
/// # Errors
///
/// [`Error::LengthMismatch`] when the lengths differ.
pub fn relative_rms_error(reference: &[f64], candidate: &[f64]) -> Result<f64> {
    check_lengths(reference, candidate)?;
    let differences: Vec<f64> = reference
        .iter()
        .zip(candidate)
        .map(|(expected, actual)| actual - expected)
        .collect();
    Ok(root_mean_square(&differences) / root_mean_square(reference))
}

/// The largest `|candidate − reference|` over the samples; 0 for empty sequences.
///
/// # Errors
///
/// [`Error::LengthMismatch`] when the lengths differ.
pub fn max_absolute_difference(reference: &[f64], candidate: &[f64]) -> Result<f64> {
    check_lengths(reference, candidate)?;
    Ok(reference
        .iter()
        .zip(candidate)
        .map(|(expected, actual)| (actual - expected).abs())
        .fold(0.0, f64::max))
}

/// A linear gain in decibels, `20 · log10(gain)`.
pub fn decibels(gain: f64) -> f64 {
    20.0 * gain.log10()
}

/// The distance in decibels between two linear gains, `|dB(candidate) − dB(reference)|`.
pub fn decibel_difference(reference: f64, candidate: f64) -> f64 {
    (decibels(candidate) - decibels(reference)).abs()
}

/// Refuses two sequences of different lengths.
fn check_lengths(reference: &[f64], candidate: &[f64]) -> Result<()> {
    if reference.len() == candidate.len() {
        Ok(())
    } else {
        Err(Error::LengthMismatch {
            reference: reference.len(),
            candidate: candidate.len(),
        })
    }
}

/// A sequence length as `f64`; compared sequences stay far below 2^53 values.
#[allow(
    clippy::cast_precision_loss,
    reason = "sequence lengths stay far below 2^53, where usize to f64 is exact"
)]
fn length_to_f64(length: usize) -> f64 {
    length as f64
}
