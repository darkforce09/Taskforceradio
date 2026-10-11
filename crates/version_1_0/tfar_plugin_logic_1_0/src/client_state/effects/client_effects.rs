//! `ClientEffects`: every DSP object one sender owns, created off the audio thread.
//!
//! **Role:** own the per-key radio effects, filters and HRTF sources of one sender, the two
//! compressors and the object-interception delay buffer; create them on request and lend them to
//! the audio path, as `clientDataEffects` (`reference/tfar_1_0/ts/src/clientData.hpp:68-246`).
//!
//! **Position:** a module of the crate's `effects`; a field of [`crate::client_state::ClientData`]; filled by
//! [`ClientEffects::ensure_dsp`] from the game link worker or the service thread; borrowed by
//! `tfar_plugin_logic_1_0::voice_pipeline` on the TeamSpeak audio thread.
//!
//! **Signals and state:** the maps of DSP objects, reached only through the session lock the
//! caller holds; no lock of its own.
//!
//! **Invariants:** only [`ClientEffects::ensure_dsp`] and the resets allocate; every accessor the
//! audio path uses looks up by `&str` and allocates nothing, answering `None` for an object that
//! was never ensured. Direct speech and radio each own a compressor, so each envelope advances
//! once per frame.

use std::collections::HashMap;

use crate::radio_effects::{
    AirborneRadioEffect, LongRangeRadioEffect, PersonalRadioEffect, UnderWaterRadioEffect,
};
use tfar_audio::dsp_filters::StereoFilter;
use tfar_audio::dynamics::{Compressor, CompressorSettings};
use tfar_audio::hrtf::HrtfSource;
use tfar_audio::sample_buffer::StereoBuffer;

use crate::client_state::effects::dsp_requirement::{DspContext, DspRequirement};
use crate::client_state::effects::filter_designs::{
    CANT_SPEAK_LOW_PASS, PHONE_BAND_PASS, SPEAKER_BAND_PASS, stereo, vehicle_low_pass,
};
use crate::client_state::effects::object_interception::{
    ObjectInterceptionFilter, ObjectInterceptionFilters,
};
use crate::client_state::error::{Error, Result};

#[cfg(test)]
#[path = "tests/client_effects.rs"]
mod tests;

/// The two compressors of a sender.
#[derive(Debug)]
struct VoiceCompressors {
    direct_speech: Compressor,
    radio: Compressor,
}

/// Every DSP object one sender owns.
#[derive(Debug, Default)]
pub struct ClientEffects {
    personal_radios: HashMap<String, PersonalRadioEffect>,
    long_range_radios: HashMap<String, LongRangeRadioEffect>,
    airborne_radios: HashMap<String, AirborneRadioEffect>,
    underwater_radios: HashMap<String, UnderWaterRadioEffect>,
    hrtf_sources: HashMap<String, HrtfSource>,
    cant_speak_filters: HashMap<String, StereoFilter>,
    /// Per sound key, per loss (its `f64` bits), as the original keys `key + to_string(loss)`.
    vehicle_filters: HashMap<String, HashMap<u64, StereoFilter>>,
    speaker_filters: HashMap<String, StereoFilter>,
    phone_filters: HashMap<String, StereoFilter>,
    object_interception: ObjectInterceptionFilters,
    compressors: Option<VoiceCompressors>,
    object_interception_buffer: StereoBuffer,
}

impl ClientEffects {
    /// No DSP object yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates the compressors, the object-interception buffer capacity and every object of
    /// `requirements` that does not exist yet; existing objects keep their state. Allocates and
    /// designs filters: call it off the audio thread. Returns how many objects it created.
    ///
    /// Every requirement is attempted; one that fails does not stop the others.
    ///
    /// # Errors
    ///
    /// The first failure: [`Error::NoHrirLibrary`] for an HRTF source without a sphere,
    /// [`Error::Filter`], [`Error::RadioEffect`] or [`Error::Compressor`] when a design is
    /// refused.
    pub fn ensure_dsp(
        &mut self,
        requirements: &[DspRequirement<'_>],
        context: &DspContext<'_>,
    ) -> Result<usize> {
        let mut created = 0_usize;
        let mut first_error = None;
        let mut record = |outcome: Result<bool>| match outcome {
            Ok(true) => created = created.saturating_add(1),
            Ok(false) => {}
            Err(error) => {
                first_error.get_or_insert(error);
            }
        };
        record(self.ensure_compressors());
        if self.object_interception_buffer.capacity_frames() < context.frames_per_block {
            self.object_interception_buffer
                .reserve_frames(context.frames_per_block);
        }
        for requirement in requirements {
            record(self.ensure_one(*requirement, context));
        }
        first_error.map_or(Ok(created), Err)
    }

    /// Creates the object `requirement` names unless it exists; whether it created it.
    fn ensure_one(
        &mut self,
        requirement: DspRequirement<'_>,
        context: &DspContext<'_>,
    ) -> Result<bool> {
        match requirement {
            DspRequirement::PersonalRadio(key) => ensure(&mut self.personal_radios, key, || {
                Ok(PersonalRadioEffect::new()?)
            }),
            DspRequirement::LongRangeRadio(key) => ensure(&mut self.long_range_radios, key, || {
                Ok(LongRangeRadioEffect::new()?)
            }),
            DspRequirement::AirborneRadio(key) => ensure(&mut self.airborne_radios, key, || {
                Ok(AirborneRadioEffect::new()?)
            }),
            DspRequirement::UnderWaterRadio(key) => {
                ensure(&mut self.underwater_radios, key, || {
                    Ok(UnderWaterRadioEffect::new(context.dropout_seed)?)
                })
            }
            DspRequirement::Hrtf(key) => ensure(&mut self.hrtf_sources, key, || {
                let library = context.hrir_library.ok_or_else(|| Error::NoHrirLibrary {
                    key: key.to_owned(),
                })?;
                Ok(HrtfSource::new(library))
            }),
            DspRequirement::CantSpeakFilter(key) => {
                ensure(&mut self.cant_speak_filters, key, || {
                    stereo(CANT_SPEAK_LOW_PASS)
                })
            }
            DspRequirement::VehicleFilter {
                key,
                vehicle_volume_loss,
            } => {
                let by_loss = match self.vehicle_filters.get_mut(key) {
                    Some(by_loss) => by_loss,
                    None => self.vehicle_filters.entry(key.to_owned()).or_default(),
                };
                let loss_key = vehicle_volume_loss.to_bits();
                if by_loss.contains_key(&loss_key) {
                    return Ok(false);
                }
                by_loss.insert(loss_key, stereo(vehicle_low_pass(vehicle_volume_loss))?);
                Ok(true)
            }
            DspRequirement::SpeakerFilter(key) => {
                ensure(&mut self.speaker_filters, key, || stereo(SPEAKER_BAND_PASS))
            }
            DspRequirement::PhoneFilter(key) => {
                ensure(&mut self.phone_filters, key, || stereo(PHONE_BAND_PASS))
            }
            DspRequirement::ObjectInterceptionFilter { object_count } => self
                .object_interception
                .ensure(object_count, context.object_interception_strength),
        }
    }

    /// Creates the two compressors unless they exist.
    fn ensure_compressors(&mut self) -> Result<bool> {
        if self.compressors.is_some() {
            return Ok(false);
        }
        self.compressors = Some(VoiceCompressors {
            direct_speech: Compressor::new(CompressorSettings::VERSION_1_0)?,
            radio: Compressor::new(CompressorSettings::VERSION_1_0)?,
        });
        Ok(true)
    }

    /// The short-range radio effect of `key`.
    pub fn personal_radio_mut(&mut self, key: &str) -> Option<&mut PersonalRadioEffect> {
        self.personal_radios.get_mut(key)
    }

    /// The long-range radio effect of `key`.
    pub fn long_range_radio_mut(&mut self, key: &str) -> Option<&mut LongRangeRadioEffect> {
        self.long_range_radios.get_mut(key)
    }

    /// The airborne radio effect of `key`.
    pub fn airborne_radio_mut(&mut self, key: &str) -> Option<&mut AirborneRadioEffect> {
        self.airborne_radios.get_mut(key)
    }

    /// The underwater radio effect of `key`.
    pub fn underwater_radio_mut(&mut self, key: &str) -> Option<&mut UnderWaterRadioEffect> {
        self.underwater_radios.get_mut(key)
    }

    /// The HRTF source of `key`.
    pub fn hrtf_source_mut(&mut self, key: &str) -> Option<&mut HrtfSource> {
        self.hrtf_sources.get_mut(key)
    }

    /// The can't-speak low-pass of `key`.
    pub fn cant_speak_filter_mut(&mut self, key: &str) -> Option<&mut StereoFilter> {
        self.cant_speak_filters.get_mut(key)
    }

    /// The vehicle low-pass of `key` at `vehicle_volume_loss`.
    pub fn vehicle_filter_mut(
        &mut self,
        key: &str,
        vehicle_volume_loss: f64,
    ) -> Option<&mut StereoFilter> {
        self.vehicle_filters
            .get_mut(key)?
            .get_mut(&vehicle_volume_loss.to_bits())
    }

    /// The loudspeaker band-pass of `key`.
    pub fn speaker_filter_mut(&mut self, key: &str) -> Option<&mut StereoFilter> {
        self.speaker_filters.get_mut(key)
    }

    /// The phone band-pass of `key`.
    pub fn phone_filter_mut(&mut self, key: &str) -> Option<&mut StereoFilter> {
        self.phone_filters.get_mut(key)
    }

    /// The object-interception low-pass for `object_count` objects, at the strength it was
    /// ensured with.
    pub fn object_interception_filter_mut(
        &mut self,
        object_count: i32,
    ) -> Option<ObjectInterceptionFilter<'_>> {
        self.object_interception.get_mut(object_count)
    }

    /// The `objectInterceptionStrength` the object-interception low-passes are designed for.
    pub const fn object_interception_strength(&self) -> Option<f32> {
        self.object_interception.strength()
    }

    /// The direct-speech compressor.
    pub fn direct_speech_compressor_mut(&mut self) -> Option<&mut Compressor> {
        self.compressors
            .as_mut()
            .map(|compressors| &mut compressors.direct_speech)
    }

    /// The radio compressor.
    pub fn radio_compressor_mut(&mut self) -> Option<&mut Compressor> {
        self.compressors
            .as_mut()
            .map(|compressors| &mut compressors.radio)
    }

    /// The previous frame of direct speech, held back one frame for the object-interception
    /// filter (`OISampleBuffer`).
    pub const fn object_interception_buffer_mut(&mut self) -> &mut StereoBuffer {
        &mut self.object_interception_buffer
    }

    /// Drops the radio effects and the speaker and phone filters (`resetRadioEffect`, `/tfar
    /// rstflt`); the airborne effects go too, which the original forgets.
    pub fn reset_radio_effects(&mut self) {
        self.personal_radios.clear();
        self.long_range_radios.clear();
        self.airborne_radios.clear();
        self.underwater_radios.clear();
        self.speaker_filters.clear();
        self.phone_filters.clear();
    }

    /// Clears the memory of the can't-speak and vehicle low-passes (`resetVoices`), as a mute
    /// does, and keeps them: a reset runs under the session lock the audio thread waits for and
    /// can land while a sender's voice still plays, where a dropped filter would leave its frames
    /// unfiltered until the next ensure.
    pub fn reset_voices(&mut self) {
        for filter in self.cant_speak_filters.values_mut() {
            filter.reset();
        }
        for filter in self
            .vehicle_filters
            .values_mut()
            .flat_map(HashMap::values_mut)
        {
            filter.reset();
        }
    }
}

/// Inserts `make()` under `key` unless the key is present; whether it inserted.
fn ensure<T>(
    map: &mut HashMap<String, T>,
    key: &str,
    make: impl FnOnce() -> Result<T>,
) -> Result<bool> {
    if map.contains_key(key) {
        return Ok(false);
    }
    map.insert(key.to_owned(), make()?);
    Ok(true)
}
