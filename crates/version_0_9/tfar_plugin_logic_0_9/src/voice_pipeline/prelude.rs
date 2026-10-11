//! The names a caller of `tfar_plugin_logic_0_9::voice_pipeline` imports with `use crate::voice_pipeline::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::voice_pipeline::distance_gain::{radio_volume_multiplier, volume_from_distance};
pub use crate::voice_pipeline::own_voice::{OWN_VOICE_STREAM_NAME, append_own_voice};
pub use crate::voice_pipeline::remote_tangent_sound::{
    ListenerView, RemoteSoundPlan, render_remote_tangent_sound,
};
pub use crate::voice_pipeline::voice_context::{OwnVoiceContext, VoiceFrameContext, VoiceOutcome};
pub use crate::voice_pipeline::voice_processor::VoiceProcessor;
pub use crate::voice_pipeline::voice_scratch::{ScratchCapacities, VoiceScratch};
