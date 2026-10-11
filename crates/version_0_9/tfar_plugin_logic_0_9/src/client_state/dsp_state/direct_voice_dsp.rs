//! `DirectVoiceDsp`: the sound state of a sender's direct voice.

use tfar_audio::dsp_filters::StereoFilter;
use tfar_audio::hrtf::{HrirLibrary, HrtfSource};

use super::dsp_requirements::DirectVoiceRequirement;
use super::filter_designs::{CANT_SPEAK_LOW_PASS, stereo_filter};
use super::vehicle_filter::{VehicleFilter, ensure_vehicle_filter};
use crate::client_state::error::Result;

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

    /// Clears the memory of both low-passes (`resetVoices`) and keeps every object, so a reset
    /// neither frees nor rebuilds anything and the next frame still finds its filters. The
    /// direction source is left to its own idle restart: reset from a command thread it could
    /// cut into a frame the audio thread is still playing.
    pub(crate) fn reset(&mut self) {
        if let Some(filter) = &mut self.cant_speak_filter {
            filter.reset();
        }
        if let Some(filter) = &mut self.vehicle_filter {
            filter.reset();
        }
    }
}
