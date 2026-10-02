//! The DSPFilters oracle: the original Butterworth and RBJ filters, run over test signals.
//!
//! **Role:** names the TFAR filter designs as DSPFilters sets them up, checks their parameters
//! and runs them through the shim: mono and stereo `f32` processing, the stereo `i16` path of the
//! original `processFilterStereo`, and the magnitude response.
//!
//! **Position:** inside `oracles`; called by the filter comparison tests, which run the same
//! design through `tfar_dsp_filters`. The design classes are DSPFilters'
//! `SimpleFilter<Butterworth::LowPass<N>>`, `SimpleFilter<Butterworth::BandPass<N>>`,
//! `SimpleFilter<RBJ::LowPass>` and `SimpleFilter<RBJ::HighPass>` with the default
//! `DirectFormII` state, from `reference/tfar_1_0/ts/include/dspfilters/`.
//!
//! **Signals and state:** none kept: every processing call sets up a fresh filter with silent
//! state, runs the whole buffer through it and drops it. Any thread may call.
//!
//! **Invariants:** a parameter outside the range DSPFilters handles is an [`Error`] before the
//! shim is called; the order of a Butterworth design is its template argument, as in the
//! originals (`LowPass<2>` set up with order 2).

use crate::error::{Error, Result};
use crate::oracles::oracle_bindings::{
    self, KIND_BUTTERWORTH_BAND_PASS, KIND_BUTTERWORTH_LOW_PASS, KIND_RBJ_HIGH_PASS,
    KIND_RBJ_LOW_PASS, ShimFilterDesign,
};
use crate::oracles::parameter_checks::{check_below_nyquist, check_positive};

#[cfg(test)]
#[path = "tests/filter_oracle.rs"]
mod tests;

/// The highest Butterworth order the oracle instantiates.
pub const ORACLE_MAX_BUTTERWORTH_ORDER: usize = 8;

/// One DSPFilters design and its `setup` parameters. Frequencies are in hertz.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OracleFilterDesign {
    /// `Butterworth::LowPass<order>` set up with `(order, rate, cutoff_hz)`.
    ButterworthLowPass {
        /// The number of poles, `1..=ORACLE_MAX_BUTTERWORTH_ORDER`.
        order: usize,
        /// The −3 dB frequency.
        cutoff_hz: f64,
    },
    /// `Butterworth::BandPass<order>` set up with `(order, rate, centre_hz, width_hz)`.
    ButterworthBandPass {
        /// The order of the low-pass prototype, `1..=ORACLE_MAX_BUTTERWORTH_ORDER`.
        order: usize,
        /// The centre frequency.
        centre_hz: f64,
        /// The width between the band edges.
        width_hz: f64,
    },
    /// `RBJ::LowPass` set up with `(rate, cutoff_hz, quality)`.
    RbjLowPass {
        /// The corner frequency.
        cutoff_hz: f64,
        /// The quality factor.
        quality: f64,
    },
    /// `RBJ::HighPass` set up with `(rate, cutoff_hz, quality)`.
    RbjHighPass {
        /// The corner frequency.
        cutoff_hz: f64,
        /// The quality factor.
        quality: f64,
    },
}

/// A checked DSPFilters design at a sample rate, ready to run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OracleFilter {
    shim_design: ShimFilterDesign,
}

impl OracleFilter {
    /// Checks `design` at `sample_rate_hz`.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidParameter`] for a sample rate that is not finite and above 0, an order
    /// outside `1..=ORACLE_MAX_BUTTERWORTH_ORDER`, a frequency not strictly between 0 and Nyquist,
    /// or a width or quality that is not finite and above 0.
    pub fn new(design: OracleFilterDesign, sample_rate_hz: f64) -> Result<Self> {
        check_positive("sample_rate_hz", sample_rate_hz)?;
        let nyquist_hz = sample_rate_hz / 2.0;
        let shim_design = match design {
            OracleFilterDesign::ButterworthLowPass { order, cutoff_hz } => {
                check_below_nyquist("cutoff_hz", cutoff_hz, nyquist_hz)?;
                ShimFilterDesign {
                    kind: KIND_BUTTERWORTH_LOW_PASS,
                    order: checked_order(order)?,
                    sample_rate_hz,
                    frequency_hz: cutoff_hz,
                    ..ShimFilterDesign::default()
                }
            }
            OracleFilterDesign::ButterworthBandPass {
                order,
                centre_hz,
                width_hz,
            } => {
                check_below_nyquist("centre_hz", centre_hz, nyquist_hz)?;
                check_positive("width_hz", width_hz)?;
                ShimFilterDesign {
                    kind: KIND_BUTTERWORTH_BAND_PASS,
                    order: checked_order(order)?,
                    sample_rate_hz,
                    frequency_hz: centre_hz,
                    width_hz,
                    ..ShimFilterDesign::default()
                }
            }
            OracleFilterDesign::RbjLowPass { cutoff_hz, quality }
            | OracleFilterDesign::RbjHighPass { cutoff_hz, quality } => {
                check_below_nyquist("cutoff_hz", cutoff_hz, nyquist_hz)?;
                check_positive("quality", quality)?;
                let kind = if matches!(design, OracleFilterDesign::RbjLowPass { .. }) {
                    KIND_RBJ_LOW_PASS
                } else {
                    KIND_RBJ_HIGH_PASS
                };
                ShimFilterDesign {
                    kind,
                    sample_rate_hz,
                    frequency_hz: cutoff_hz,
                    quality,
                    ..ShimFilterDesign::default()
                }
            }
        };
        Ok(Self { shim_design })
    }

    /// Runs mono `f32` samples in place through `SimpleFilter<Design, 1>::process<float>`, from
    /// silent state.
    ///
    /// # Errors
    ///
    /// [`Error::OracleRefused`] when the shim refuses the call.
    pub fn process_mono_f32(&self, samples: &mut [f32]) -> Result<()> {
        oracle_bindings::filter_process_f32(&self.shim_design, 1, samples)
    }

    /// Runs interleaved stereo `f32` frames in place through
    /// `SimpleFilter<Design, 2>::process<float>`, from silent state.
    ///
    /// # Errors
    ///
    /// [`Error::OracleRefused`] when the shim refuses the call.
    pub fn process_stereo_f32(&self, frames: &mut [[f32; 2]]) -> Result<()> {
        oracle_bindings::filter_process_f32(&self.shim_design, 2, frames.as_flattened_mut())
    }

    /// Filters interleaved stereo `i16` samples in place as `processFilterStereo`
    /// (`reference/tfar_1_0/ts/src/helpers.hpp`) does, multiplying by `gain` before the
    /// conversion back, from silent state.
    ///
    /// # Errors
    ///
    /// [`Error::UnevenFrames`] for an odd number of samples; [`Error::OracleRefused`] when the
    /// shim refuses the call.
    pub fn process_stereo_i16(&self, samples: &mut [i16], gain: f32) -> Result<()> {
        oracle_bindings::filter_process_stereo_i16(&self.shim_design, samples, gain)
    }

    /// The linear gain `|H|` DSPFilters' `response` gives at each of `frequencies_hz`.
    ///
    /// # Errors
    ///
    /// [`Error::OracleRefused`] when the shim refuses the call.
    pub fn magnitudes(&self, frequencies_hz: &[f64]) -> Result<Vec<f64>> {
        let mut magnitudes = vec![0.0; frequencies_hz.len()];
        oracle_bindings::filter_magnitudes(&self.shim_design, frequencies_hz, &mut magnitudes)?;
        Ok(magnitudes)
    }
}

/// The order as the shim's `int`, checked against the instantiated range.
fn checked_order(order: usize) -> Result<i32> {
    match i32::try_from(order) {
        Ok(shim_order) if (1..=ORACLE_MAX_BUTTERWORTH_ORDER).contains(&order) => Ok(shim_order),
        _ => Err(Error::InvalidParameter {
            parameter: "order",
            value: order_as_value(order),
            requirement: "within 1 to ORACLE_MAX_BUTTERWORTH_ORDER",
        }),
    }
}

/// An order as the `f64` an [`Error::InvalidParameter`] carries.
#[allow(
    clippy::cast_precision_loss,
    reason = "the order only labels the error; any rounding of a huge order is harmless"
)]
fn order_as_value(order: usize) -> f64 {
    order as f64
}
