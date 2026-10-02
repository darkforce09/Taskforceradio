//! `VehicleFilter`: the vehicle low-pass of one sound, designed for the isolation it was built at.

use tfar_dsp_filters::StereoFilter;

use super::filter_designs::{stereo_filter, vehicle_low_pass};
use crate::error::Result;

/// A vehicle low-pass and the combined isolation it was designed for.
///
/// The original keeps one filter per key and isolation (`key + to_string(loss)`) and frees them
/// only when the voices reset; here one filter per key is redesigned when the isolation changes,
/// so the state never grows with the isolations a sound has passed through.
#[derive(Debug, Clone)]
pub struct VehicleFilter {
    isolation: f64,
    filter: StereoFilter,
}

impl VehicleFilter {
    /// A silent low-pass for `isolation`. Allocates and designs: call it off the audio thread.
    ///
    /// # Errors
    ///
    /// [`crate::Error::FilterDesign`] when the filter library refuses the design.
    pub(crate) fn new(isolation: f64) -> Result<Self> {
        Ok(Self {
            isolation,
            filter: stereo_filter("vehicle low-pass", vehicle_low_pass(isolation))?,
        })
    }

    /// The combined isolation the filter is designed for.
    pub fn isolation(&self) -> f64 {
        self.isolation
    }

    /// The filter, to run on the audio thread.
    pub fn filter_mut(&mut self) -> &mut StereoFilter {
        &mut self.filter
    }
}

/// Makes `slot` hold a vehicle filter for `isolation`: kept when it already has that isolation,
/// built otherwise. Allocates only when it builds.
///
/// # Errors
///
/// [`crate::Error::FilterDesign`] when the filter library refuses the design.
pub(crate) fn ensure_vehicle_filter(
    slot: &mut Option<VehicleFilter>,
    isolation: f64,
) -> Result<()> {
    let is_current = slot
        .as_ref()
        .is_some_and(|existing| existing.isolation.to_bits() == isolation.to_bits());
    if !is_current {
        *slot = Some(VehicleFilter::new(isolation)?);
    }
    Ok(())
}
