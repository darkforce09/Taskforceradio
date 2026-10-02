//! The 0.9.12 audio path: one TeamSpeak voice frame through routing, effects, HRTF and mixing.
//!
//! **Role:** the 0.9.12 stereo function per sender: the gates, the voice volume multiplier, the
//! direct voice with its distance gain, vehicle and can't-speak branches and its direction, one
//! compressor pass, every radio reception with its effect and loudspeaker stages, and the global
//! volume; plus the local player's own voice for loudspeakers.
//!
//! **Position:** `crates/version_0_9`, tier 5, depending on `tfar_client_state_0_9`,
//! `tfar_radio_routing_0_9`, `tfar_radio_effects_0_9`, `tfar_dsp_filters`, `tfar_dynamics`,
//! `tfar_hrtf`, `tfar_spatial_audio`, `tfar_sample_buffer`, `tfar_playback_mixer`; used by
//! `tfar_plugin_0_9`.
//!
//! **Signals and state:** a [`VoiceProcessor`] per server connection owns its scratch; the sound
//! state it advances belongs to the client entries. It runs on the TeamSpeak audio thread with
//! the server state locked by the plugin, makes no TeamSpeak call and makes or drops no sound
//! object.
//!
//! **Invariants:** no frame within the scratch's size changes a scratch capacity; the compressor
//! runs once per frame; a path whose sound object is missing stays silent; output saturates.

mod crate_role;
mod distance_gain;
mod error;
mod own_voice;
pub mod prelude;
mod voice_chain;
mod voice_context;
mod voice_gains;
mod voice_processor;
mod voice_scratch;

#[cfg(test)]
#[path = "tests/test_scene.rs"]
mod test_scene;

pub use crate_role::CRATE_ROLE;
pub use distance_gain::{
    CANT_SPEAK_DISTANCE, MINIMUM_AUDIBLE_GAIN, radio_volume_multiplier, volume_from_distance,
};
pub use error::{Error, Result};
pub use own_voice::{OWN_VOICE_STREAM_NAME, append_own_voice};
pub use voice_context::{OwnVoiceContext, VoiceFrameContext, VoiceOutcome};
pub use voice_gains::{
    CANT_SPEAK_GAIN, DIVER_RADIO_EFFECT_GAIN, HRTF_MAKEUP_GAIN_DB, PHONE_GAIN, RADIO_EFFECT_GAIN,
    SPEAKER_GAIN, VEHICLE_LOSS_EXPONENT, hrtf_makeup_gain, vehicle_loss_gain,
};
pub use voice_processor::VoiceProcessor;
pub use voice_scratch::{
    DEFAULT_MAX_FRAMES, NICKNAME_CAPACITY, ROUTE_CAPACITY, ScratchCapacities, VoiceScratch,
};
