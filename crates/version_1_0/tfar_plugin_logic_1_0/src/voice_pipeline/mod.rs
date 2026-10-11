//! The 1.0 audio path: one TeamSpeak voice frame through routing, effects, compressor, HRTF and
//! mixing.
//!
//! **Role:** the 1.0 audio path for one voice frame: gates the sender, applies direct speech
//! (muffling, the distance curve and voice cone level, HRTF direction, its compressor) and every
//! radio reception (the radio compressor, the effects, speakers, intercom), then the global
//! volume; creates the DSP objects those frames need, off the audio thread; and shapes the
//! positional remote tangent sound.
//!
//! **Position:** a module of `tfar_plugin_logic_1_0`, depending on `tfar_plugin_logic_1_0::client_state`,
//! `tfar_plugin_logic_1_0::radio_routing`, `tfar_plugin_logic_1_0::radio_effects`, `tfar_plugin_logic_1_0::plugin_settings`,
//! `tfar_plugin_logic_1_0::antennas`, `tfar_foundation::geometry`, `tfar_audio::dsp_filters`, `tfar_audio::dynamics`, `tfar_audio::hrtf`,
//! `tfar_audio::spatial_audio`, `tfar_audio::sample_buffer`, `tfar_audio::playback_mixer`; used by
//! `tfar_plugin_1_0`.
//!
//! **Signals and state:** a [`VoiceProcessor`] per server connection with its scratch buffers;
//! the DSP objects live in the senders' client state. Called on the TeamSpeak audio threads
//! with the server lock held; [`ensure_sender_dsp`] and [`render_remote_tangent_sound`] run off
//! them.
//!
//! **Invariants:** each compressor runs once per frame; nothing pans twice; the audio path
//! allocates nothing once its scratch fits the frame.

mod direct_speech;
mod distance_gain;
mod dsp_needs;
mod error;
mod filter_step;
mod frame_plan;
mod placement;
pub mod prelude;
mod radio_paths;
mod remote_tangent_sound;
mod scene;
mod sender_gate;
mod voice_context;
mod voice_processor;
mod voice_scratch;

pub use distance_gain::{
    CANT_SPEAK_DISTANCE, CANT_SPEAK_GAIN, MAX_VEHICLE_LOSS, NO_VEHICLE_LOSS, SPEAKER_GAIN,
    UNDERWATER_LEVEL, speaker_audible_distance, vehicle_loss_gain, vehicle_volume_loss,
    volume_attenuation,
};
pub use dsp_needs::{ensure_every_sender_dsp, ensure_sender_dsp};
pub use error::{Error, Result};
pub use placement::{HRTF_MAKEUP_GAIN_DB, hrtf_makeup_gain};
pub use remote_tangent_sound::{
    RemoteSoundListener, RemoteSoundPlan, UNDER_WATER_SPEAKER_GAIN, apply_stereo_mode,
    render_remote_tangent_sound,
};
pub use voice_context::{VoiceFrameContext, VoiceOutcome};
pub use voice_processor::{OWN_RADIO_VOICE_STREAM, VoiceProcessor};
pub use voice_scratch::VoiceScratchCapacities;
