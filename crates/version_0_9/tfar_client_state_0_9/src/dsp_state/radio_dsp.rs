//! `RadioDsp`: the sound state of a sender heard over one radio.
//!
//! **Role:** hold, for one radio id of one sender, the radio effect of each family and the
//! phone, loudspeaker, under-water and vehicle filters and the direction source, made on
//! request and dropped by the two resets.
//!
//! **Position:** owned by `ClientDsp`, keyed by radio id; filled by `ClientDsp::ensure` from a
//! `RadioRequirement`, read by the voice pipeline.
//!
//! **Signals and state:** the effect and filter memories. Made and dropped on the command
//! threads under the server lock; the audio thread only borrows them through the `_mut`
//! accessors and never makes one.
//!
//! **Invariants:** `reset_radio_effects` drops exactly the effects and the loudspeaker and phone
//! filters (`resetRadioEffect`, `reference/tfar_0_9_12/ts/src/plugin.cpp:319`); `reset_voices`
//! drops exactly the direction source and the under-water and vehicle filters (`resetVoices`,
//! plugin.cpp:333).

use tfar_dsp_filters::StereoFilter;
use tfar_hrtf::{HrirLibrary, HrtfSource};
use tfar_radio_effects_0_9::{LongRangeRadioEffect, PersonalRadioEffect, UnderWaterRadioEffect};

use super::dsp_requirements::{RadioEffectKind, RadioRequirement};
use super::filter_designs::{
    CANT_SPEAK_LOW_PASS, PHONE_BAND_PASS, SPEAKER_BAND_PASS, stereo_filter,
};
use super::vehicle_filter::{VehicleFilter, ensure_vehicle_filter};
use crate::error::Result;

/// The state of one sender heard over one radio id.
#[derive(Debug, Default)]
pub struct RadioDsp {
    personal_effect: Option<PersonalRadioEffect>,
    long_range_effect: Option<LongRangeRadioEffect>,
    under_water_effect: Option<UnderWaterRadioEffect>,
    phone_filter: Option<StereoFilter>,
    speaker_filter: Option<StereoFilter>,
    under_water_filter: Option<StereoFilter>,
    vehicle_filter: Option<VehicleFilter>,
    hrtf_source: Option<HrtfSource>,
}

impl RadioDsp {
    /// The personal radio effect (`digital`), when one was made.
    pub fn personal_effect_mut(&mut self) -> Option<&mut PersonalRadioEffect> {
        self.personal_effect.as_mut()
    }

    /// The long-range radio effect (`digital_lr`, `airborne`), when one was made.
    pub fn long_range_effect_mut(&mut self) -> Option<&mut LongRangeRadioEffect> {
        self.long_range_effect.as_mut()
    }

    /// The underwater radio effect (`dd`), when one was made.
    pub fn under_water_effect_mut(&mut self) -> Option<&mut UnderWaterRadioEffect> {
        self.under_water_effect.as_mut()
    }

    /// The phone band-pass (`phone`), when one was made.
    pub fn phone_filter_mut(&mut self) -> Option<&mut StereoFilter> {
        self.phone_filter.as_mut()
    }

    /// The loudspeaker band-pass, when one was made.
    pub fn speaker_filter_mut(&mut self) -> Option<&mut StereoFilter> {
        self.speaker_filter.as_mut()
    }

    /// The low-pass of a loudspeaker radio under water, when one was made.
    pub fn under_water_filter_mut(&mut self) -> Option<&mut StereoFilter> {
        self.under_water_filter.as_mut()
    }

    /// The vehicle low-pass between the listener and the loudspeaker radio, when one was made.
    pub fn vehicle_filter_mut(&mut self) -> Option<&mut VehicleFilter> {
        self.vehicle_filter.as_mut()
    }

    /// The direction source of the loudspeaker radio, when one was made.
    pub fn hrtf_source_mut(&mut self) -> Option<&mut HrtfSource> {
        self.hrtf_source.as_mut()
    }

    /// Whether any object is held.
    pub fn is_empty(&self) -> bool {
        self.personal_effect.is_none()
            && self.long_range_effect.is_none()
            && self.under_water_effect.is_none()
            && self.phone_filter.is_none()
            && self.speaker_filter.is_none()
            && self.under_water_filter.is_none()
            && self.vehicle_filter.is_none()
            && self.hrtf_source.is_none()
    }

    /// Makes every object `requirement` names; the direction source only when `hrtf` is given.
    /// An underwater effect draws its dropouts from `dropout_seed`.
    pub(crate) fn ensure(
        &mut self,
        requirement: &RadioRequirement,
        hrtf: Option<&HrirLibrary>,
        dropout_seed: u64,
    ) -> Result<()> {
        match requirement.effect {
            RadioEffectKind::Personal if self.personal_effect.is_none() => {
                self.personal_effect = Some(PersonalRadioEffect::new()?);
            }
            RadioEffectKind::LongRange if self.long_range_effect.is_none() => {
                self.long_range_effect = Some(LongRangeRadioEffect::new()?);
            }
            RadioEffectKind::UnderWater if self.under_water_effect.is_none() => {
                self.under_water_effect = Some(UnderWaterRadioEffect::new(dropout_seed)?);
            }
            RadioEffectKind::Phone if self.phone_filter.is_none() => {
                self.phone_filter = Some(stereo_filter("phone band-pass", PHONE_BAND_PASS)?);
            }
            _ => {}
        }
        let Some(speaker) = requirement.speaker else {
            return Ok(());
        };
        if self.speaker_filter.is_none() {
            self.speaker_filter = Some(stereo_filter("loudspeaker band-pass", SPEAKER_BAND_PASS)?);
        }
        if speaker.is_under_water && self.under_water_filter.is_none() {
            self.under_water_filter =
                Some(stereo_filter("under-water low-pass", CANT_SPEAK_LOW_PASS)?);
        }
        if let Some(isolation) = speaker.vehicle_isolation {
            ensure_vehicle_filter(&mut self.vehicle_filter, isolation)?;
        }
        if let (None, Some(library)) = (&self.hrtf_source, hrtf) {
            self.hrtf_source = Some(HrtfSource::new(library));
        }
        Ok(())
    }

    /// Drops the three radio effects and the phone and loudspeaker band-passes.
    pub(crate) fn reset_radio_effects(&mut self) {
        self.personal_effect = None;
        self.long_range_effect = None;
        self.under_water_effect = None;
        self.phone_filter = None;
        self.speaker_filter = None;
    }

    /// Drops the direction source and the under-water and vehicle low-passes.
    pub(crate) fn reset_voices(&mut self) {
        self.hrtf_source = None;
        self.under_water_filter = None;
        self.vehicle_filter = None;
    }
}
