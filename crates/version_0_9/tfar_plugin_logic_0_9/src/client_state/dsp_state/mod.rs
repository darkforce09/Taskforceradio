//! The sound state a sender's voice passes through, made on request off the audio thread.

mod client_dsp;
mod direct_voice_dsp;
mod dsp_requirements;
mod filter_designs;
mod radio_dsp;
mod vehicle_filter;

pub use client_dsp::ClientDsp;
pub use direct_voice_dsp::DirectVoiceDsp;
pub use dsp_requirements::{
    DirectVoiceRequirement, DspRequirements, RadioEffectKind, RadioRequirement, SpeakerRequirement,
};
pub use filter_designs::{
    CANT_SPEAK_LOW_PASS, FILTER_SAMPLE_RATE_HZ, PHONE_BAND_PASS, SPEAKER_BAND_PASS,
    vehicle_low_pass,
};
pub use radio_dsp::RadioDsp;
pub use vehicle_filter::VehicleFilter;
