//! `DspRequirement` and `DspContext`: what [`ClientEffects::ensure_dsp`] creates, and with what.
//!
//! [`ClientEffects::ensure_dsp`]: crate::client_state::ClientEffects::ensure_dsp

use tfar_audio::hrtf::HrirLibrary;

/// One DSP object a sender needs, named by the key the original stores it under
/// (`clientDataEffects`, `reference/tfar_1_0/ts/src/clientData.hpp:68-246`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DspRequirement<'a> {
    /// The short-range radio effect of the radio `key` (`getSwRadioEffect`).
    PersonalRadio(&'a str),
    /// The long-range radio effect of the radio `key`, the intercom's too (`getLrRadioEffect`).
    LongRangeRadio(&'a str),
    /// The airborne radio effect of the radio `key` (`getAirborneRadioEffect`).
    AirborneRadio(&'a str),
    /// The underwater radio effect of the radio `key` (`getUnderwaterRadioEffect`).
    UnderWaterRadio(&'a str),
    /// The HRTF source `key`: [`crate::client_state::VOICE_HRTF_KEY`] for direct speech, a radio id for a
    /// speaker (`getClunk`).
    Hrtf(&'a str),
    /// The 100 Hz low-pass of the sound `key` (`getFilterCantSpeak`).
    CantSpeakFilter(&'a str),
    /// The vehicle low-pass of the sound `key` at `vehicle_volume_loss` (`getFilterVehicle`).
    VehicleFilter {
        /// The sound the filter belongs to.
        key: &'a str,
        /// The isolation loss, 0 to 0.99, which sets the cutoff.
        vehicle_volume_loss: f64,
    },
    /// The loudspeaker band-pass of the speaker radio `key` (`getSpeakerFilter`).
    SpeakerFilter(&'a str),
    /// The phone band-pass of the radio `key` (`getSpeakerPhone`).
    PhoneFilter(&'a str),
    /// The object-interception low-pass for `object_count` objects
    /// (`getFilterObjectInterception`).
    ObjectInterceptionFilter {
        /// The objects between sender and listener, as `POS` reports them.
        object_count: i32,
    },
}

/// What [`crate::client_state::ClientEffects::ensure_dsp`] builds the DSP objects with.
#[derive(Clone, Copy, Debug)]
pub struct DspContext<'a> {
    /// The HRIR sphere; `None` when it is missing and HRTF is off.
    pub hrir_library: Option<&'a HrirLibrary>,
    /// The `objectInterceptionStrength` setting.
    pub object_interception_strength: f32,
    /// The seed of the next underwater effect's dropout generator.
    pub dropout_seed: u64,
    /// The largest block, in frames, the audio callbacks hand over; the object-interception
    /// buffer reserves it so the audio path never grows it.
    pub frames_per_block: usize,
}
