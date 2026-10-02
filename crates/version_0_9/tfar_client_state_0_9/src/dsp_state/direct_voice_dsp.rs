//! `DirectVoiceDsp`: the sound state of a sender's direct voice.

use tfar_dsp_filters::StereoFilter;
use tfar_hrtf::{HrirLibrary, HrtfSource};

use super::dsp_requirements::DirectVoiceRequirement;
use super::filter_designs::{CANT_SPEAK_LOW_PASS, stereo_filter};
use super::vehicle_filter::{VehicleFilter, ensure_vehicle_filter};
use crate::error::Result;

/// The direct voice's state: its direction source (the original's `voice_clunk`), its
/// can't-speak low-pass (`local_cantspeak`) and its vehicle low-pass (`local_vehicle`).
#[derive(Debug, Default)]
pub struct DirectVoiceDsp {
    hrtf_source: Option<HrtfSource>,
    cant_speak_filter: Option<StereoFilter>,
    vehicle_filter: Option<VehicleFilter>,
}

impl DirectVoiceDsp {
    /// The direction source, when one was made.
    pub fn hrtf_source_mut(&mut self) -> Option<&mut HrtfSource> {
        self.hrtf_source.as_mut()
    }

    /// The can't-speak low-pass, when one was made.
    pub fn cant_speak_filter_mut(&mut self) -> Option<&mut StereoFilter> {
        self.cant_speak_filter.as_mut()
    }

    /// The vehicle low-pass, when one was made.
    pub fn vehicle_filter_mut(&mut self) -> Option<&mut VehicleFilter> {
        self.vehicle_filter.as_mut()
    }

    /// Whether any object is held.
    pub fn is_empty(&self) -> bool {
        self.hrtf_source.is_none()
            && self.cant_speak_filter.is_none()
            && self.vehicle_filter.is_none()
    }

    /// Makes every object `requirement` names; the direction source only when `hrtf` is given.
    pub(crate) fn ensure(
        &mut self,
        requirement: &DirectVoiceRequirement,
        hrtf: Option<&HrirLibrary>,
    ) -> Result<()> {
        if let (None, Some(library)) = (&self.hrtf_source, hrtf) {
            self.hrtf_source = Some(HrtfSource::new(library));
        }
        if requirement.needs_cant_speak_filter && self.cant_speak_filter.is_none() {
            self.cant_speak_filter =
                Some(stereo_filter("can't-speak low-pass", CANT_SPEAK_LOW_PASS)?);
        }
        if let Some(isolation) = requirement.vehicle_isolation {
            ensure_vehicle_filter(&mut self.vehicle_filter, isolation)?;
        }
        Ok(())
    }

    /// Drops the direction source and both low-passes (`resetVoices`).
    pub(crate) fn reset(&mut self) {
        self.hrtf_source = None;
        self.cant_speak_filter = None;
        self.vehicle_filter = None;
    }
}
