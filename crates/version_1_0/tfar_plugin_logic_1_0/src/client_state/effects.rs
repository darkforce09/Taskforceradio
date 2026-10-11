//! The DSP objects each sender owns: radio effects, filters, HRTF sources and compressors.

mod client_effects;
mod dsp_requirement;
mod filter_designs;
mod object_interception;

pub use client_effects::ClientEffects;
pub use dsp_requirement::{DspContext, DspRequirement};
pub use filter_designs::{
    CANT_SPEAK_FILTER_KEY, CANT_SPEAK_LOW_PASS, PHONE_BAND_PASS, SPEAKER_BAND_PASS,
    VEHICLE_FILTER_KEY, VOICE_HRTF_KEY, object_interception_cutoff_hz, vehicle_low_pass,
};
pub use object_interception::ObjectInterceptionFilter;
