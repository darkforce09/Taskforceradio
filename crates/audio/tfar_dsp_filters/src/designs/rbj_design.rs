//! RBJ high-pass and low-pass designs through `biquad`, returned as one second-order section.
//!
//! The coefficients are Robert Bristow-Johnson's cookbook formulas, the same DSPFilters'
//! `RBJ::LowPass::setup` and `RBJ::HighPass::setup` compute
//! (`reference/tfar_1_0/ts/include/dspfilters/source/RBJ.cpp`): `w0 = 2π f / fs`,
//! `alpha = sin(w0) / (2 Q)`, normalised by `a0 = 1 + alpha`. Every parameter is checked first;
//! the section `Vec` allocates, so this runs off the audio thread.

use biquad::{Coefficients, Hertz, Type};

use super::parameter_checks::{check_design_frequency, check_quality, check_sample_rate};
use crate::error::{Error, Result};
use crate::second_order_sections::{SecondOrderSection, SecondOrderSections};

#[cfg(test)]
#[path = "tests/rbj_design.rs"]
mod tests;

/// The RBJ low-pass at `cutoff_hz` with quality factor `quality`.
pub(crate) fn rbj_low_pass(
    cutoff_hz: f64,
    quality: f64,
    sample_rate_hz: f64,
) -> Result<SecondOrderSections> {
    design(Type::LowPass, cutoff_hz, quality, sample_rate_hz)
}

/// The RBJ high-pass at `cutoff_hz` with quality factor `quality`.
pub(crate) fn rbj_high_pass(
    cutoff_hz: f64,
    quality: f64,
    sample_rate_hz: f64,
) -> Result<SecondOrderSections> {
    design(Type::HighPass, cutoff_hz, quality, sample_rate_hz)
}

/// Checks the parameters, asks `biquad` for the coefficients and wraps them as one section.
fn design(
    filter_type: Type<f64>,
    cutoff_hz: f64,
    quality: f64,
    sample_rate_hz: f64,
) -> Result<SecondOrderSections> {
    check_sample_rate(sample_rate_hz)?;
    check_design_frequency("cutoff", cutoff_hz, sample_rate_hz)?;
    check_quality(quality)?;
    let refused = |_| Error::DesignFailed {
        reason: "biquad refused the RBJ parameters",
    };
    let sample_rate = Hertz::<f64>::from_hz(sample_rate_hz).map_err(refused)?;
    let cutoff = Hertz::<f64>::from_hz(cutoff_hz).map_err(refused)?;
    let coefficients = Coefficients::<f64>::from_params(filter_type, sample_rate, cutoff, quality)
        .map_err(refused)?;
    let section = SecondOrderSection::from_unnormalised(
        [coefficients.b0, coefficients.b1, coefficients.b2],
        [1.0, coefficients.a1, coefficients.a2],
    )?;
    SecondOrderSections::new(vec![section], sample_rate_hz)
}
