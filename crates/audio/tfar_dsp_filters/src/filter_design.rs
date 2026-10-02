//! The filter designs TFAR uses, as one enum a caller fills in and turns into coefficients.
//!
//! **Role:** names each design with its parameters (Butterworth low-pass and band-pass, RBJ
//! low-pass and high-pass) and dispatches to the design library that computes it.
//!
//! **Position:** tier 1 of crates/audio; the radio effect, client state and voice pipeline crates
//! build their filters from it, through [`crate::ChannelFilter::new`] or
//! [`FilterDesign::second_order_sections`].
//!
//! **Signals and state:** plain values. Designing allocates and, for the Butterworth designs,
//! takes on the order of a millisecond, so it runs off the TeamSpeak audio thread: once per
//! client, radio or effect, never per frame.
//!
//! **Invariants:** a design either returns finite, normalised sections or an [`Error`]; it never
//! panics, whatever the parameters.

use crate::designs;
use crate::error::Result;
use crate::second_order_sections::SecondOrderSections;

#[cfg(test)]
#[path = "tests/filter_design.rs"]
mod tests;

/// The highest Butterworth order this crate designs. TFAR uses orders 1, 2 and 4.
pub const MAX_BUTTERWORTH_ORDER: usize = 8;

/// One filter design and its parameters. Frequencies are in hertz.
///
/// The originals' designs map as follows: DSPFilters `Butterworth::LowPass<N>` →
/// [`FilterDesign::ButterworthLowPass`] with `order: N`; `Butterworth::BandPass<N>` set up with
/// `(N, rate, centre, width)` → [`FilterDesign::ButterworthBandPass`]; `RBJ::LowPass` and
/// `RBJ::HighPass` set up with `(rate, cutoff, q)` → [`FilterDesign::RbjLowPass`] and
/// [`FilterDesign::RbjHighPass`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterDesign {
    /// A Butterworth low-pass of `order` poles with its −3 dB point at `cutoff_hz`.
    ButterworthLowPass {
        /// The number of poles, `1..=MAX_BUTTERWORTH_ORDER`.
        order: usize,
        /// The −3 dB frequency, strictly between 0 and Nyquist.
        cutoff_hz: f64,
    },
    /// A Butterworth band-pass of 2 × `order` poles, in DSPFilters' centre-and-width convention:
    /// the −3 dB band edges are `centre_hz ∓ width_hz / 2`, and the gain is 1 at the warped
    /// geometric centre between them.
    ButterworthBandPass {
        /// The order of the low-pass prototype, `1..=MAX_BUTTERWORTH_ORDER`.
        order: usize,
        /// The centre frequency.
        centre_hz: f64,
        /// The distance between the two −3 dB band edges; both edges lie strictly between 0 and
        /// Nyquist.
        width_hz: f64,
    },
    /// An RBJ cookbook low-pass at `cutoff_hz` with quality factor `quality`.
    RbjLowPass {
        /// The corner frequency, strictly between 0 and Nyquist.
        cutoff_hz: f64,
        /// The quality factor, above 0 (`1 / √2` is maximally flat).
        quality: f64,
    },
    /// An RBJ cookbook high-pass at `cutoff_hz` with quality factor `quality`.
    RbjHighPass {
        /// The corner frequency, strictly between 0 and Nyquist.
        cutoff_hz: f64,
        /// The quality factor, above 0 (`1 / √2` is maximally flat).
        quality: f64,
    },
}

impl FilterDesign {
    /// Designs the filter for `sample_rate_hz` and returns its normalised sections. Allocates and
    /// may take about a millisecond: call it off the audio thread.
    ///
    /// # Errors
    ///
    /// [`crate::Error::InvalidSampleRate`], [`crate::Error::FrequencyOutOfRange`],
    /// [`crate::Error::InvalidBandWidth`], [`crate::Error::InvalidQuality`] or
    /// [`crate::Error::InvalidOrder`] for a parameter outside its range;
    /// [`crate::Error::DesignFailed`] when the design library returns unusable coefficients.
    pub fn second_order_sections(&self, sample_rate_hz: f64) -> Result<SecondOrderSections> {
        match *self {
            Self::ButterworthLowPass { order, cutoff_hz } => {
                designs::butterworth_low_pass(order, cutoff_hz, sample_rate_hz)
            }
            Self::ButterworthBandPass {
                order,
                centre_hz,
                width_hz,
            } => designs::butterworth_band_pass(order, centre_hz, width_hz, sample_rate_hz),
            Self::RbjLowPass { cutoff_hz, quality } => {
                designs::rbj_low_pass(cutoff_hz, quality, sample_rate_hz)
            }
            Self::RbjHighPass { cutoff_hz, quality } => {
                designs::rbj_high_pass(cutoff_hz, quality, sample_rate_hz)
            }
        }
    }

    /// The −3 dB band edges `(lower, upper)` in hertz of a [`FilterDesign::ButterworthBandPass`],
    /// `centre ∓ width / 2`; `None` for every other design.
    pub fn band_edges_hz(&self) -> Option<(f64, f64)> {
        match *self {
            Self::ButterworthBandPass {
                centre_hz,
                width_hz,
                ..
            } => Some(designs::band_edges_hz(centre_hz, width_hz)),
            Self::ButterworthLowPass { .. }
            | Self::RbjLowPass { .. }
            | Self::RbjHighPass { .. } => None,
        }
    }
}
