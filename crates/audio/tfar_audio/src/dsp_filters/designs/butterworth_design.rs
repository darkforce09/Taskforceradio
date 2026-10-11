//! Butterworth low-pass and band-pass designs through `sci-rs`, returned as second-order sections.
//!
//! **Role:** turns an order and a cutoff, or an order, a centre and a width, into the normalised
//! sections of the digital Butterworth filter `scipy.signal.butter(..., output="sos")` designs.
//!
//! **Position:** called by [`crate::dsp_filters::FilterDesign::second_order_sections`]; the design itself is
//! `sci_rs::signal::filter::design::butter_dyn`.
//!
//! **Signals and state:** none; allocates the section `Vec`, so it runs off the audio thread.
//!
//! **Invariants:** every parameter is checked before `butter_dyn` is called, so none of its
//! panics is reachable. The band-pass mapping follows DSPFilters'
//! `BandPassBase::setup(order, sampleRate, centerFrequency, widthFrequency)`
//! (`reference/tfar_1_0/ts/include/dspfilters/source/PoleFilter.cpp`, `BandPassTransform`): the
//! band edges, where the gain is −3 dB, are `centre − width / 2` and `centre + width / 2`, and the
//! peak gain of 1 lies at the warped geometric centre between them. An order-N band-pass has 2N
//! poles, as DSPFilters' `BandPass<N>` has.

use sci_rs::signal::filter::design::{
    DigitalFilter, FilterBandType, FilterOutputType, Sos, butter_dyn,
};

use super::parameter_checks::{check_design_frequency, check_order, check_sample_rate};
use crate::dsp_filters::error::{Error, Result};
use crate::dsp_filters::second_order_sections::{SecondOrderSection, SecondOrderSections};

#[cfg(test)]
#[path = "tests/butterworth_design.rs"]
mod tests;

/// The lower and upper band edges, in hertz, of a band-pass given as centre and width: the
/// DSPFilters convention `centre ∓ width / 2`.
pub(crate) fn band_edges_hz(centre_hz: f64, width_hz: f64) -> (f64, f64) {
    let half_width = width_hz / 2.0;
    (centre_hz - half_width, centre_hz + half_width)
}

/// The Butterworth low-pass of `order` with its −3 dB point at `cutoff_hz`.
pub(crate) fn butterworth_low_pass(
    order: usize,
    cutoff_hz: f64,
    sample_rate_hz: f64,
) -> Result<SecondOrderSections> {
    check_order(order)?;
    check_sample_rate(sample_rate_hz)?;
    check_design_frequency("cutoff", cutoff_hz, sample_rate_hz)?;
    design(
        order,
        vec![cutoff_hz],
        FilterBandType::Lowpass,
        sample_rate_hz,
    )
}

/// The Butterworth band-pass of `order` (2 × `order` poles) between the band edges
/// [`band_edges_hz`] derives from `centre_hz` and `width_hz`.
pub(crate) fn butterworth_band_pass(
    order: usize,
    centre_hz: f64,
    width_hz: f64,
    sample_rate_hz: f64,
) -> Result<SecondOrderSections> {
    check_order(order)?;
    check_sample_rate(sample_rate_hz)?;
    if !(width_hz.is_finite() && width_hz > 0.0) {
        return Err(Error::InvalidBandWidth {
            centre_hz,
            width_hz,
        });
    }
    let (lower_hz, upper_hz) = band_edges_hz(centre_hz, width_hz);
    let lower = check_design_frequency("lower band edge", lower_hz, sample_rate_hz)?;
    let upper = check_design_frequency("upper band edge", upper_hz, sample_rate_hz)?;
    if lower >= upper {
        return Err(Error::InvalidBandWidth {
            centre_hz,
            width_hz,
        });
    }
    design(
        order,
        vec![lower_hz, upper_hz],
        FilterBandType::Bandpass,
        sample_rate_hz,
    )
}

/// Runs `butter_dyn` on checked parameters and normalises its sections.
fn design(
    order: usize,
    critical_frequencies_hz: Vec<f64>,
    band: FilterBandType,
    sample_rate_hz: f64,
) -> Result<SecondOrderSections> {
    let filter = butter_dyn(
        order,
        critical_frequencies_hz,
        Some(band),
        Some(false),
        Some(FilterOutputType::Sos),
        Some(sample_rate_hz),
    );
    let DigitalFilter::Sos(sos_filter) = filter else {
        return Err(Error::DesignFailed {
            reason: "the Butterworth design returned no second-order sections",
        });
    };
    let sections = sos_filter
        .sos
        .iter()
        .map(|section: &Sos<f64>| SecondOrderSection::from_unnormalised(section.b, section.a))
        .collect::<Result<Vec<_>>>()?;
    SecondOrderSections::new(sections, sample_rate_hz)
}
