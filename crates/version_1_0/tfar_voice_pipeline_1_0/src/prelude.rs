//! The names a caller of `tfar_voice_pipeline_1_0` imports with `use tfar_voice_pipeline_1_0::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
pub use crate::distance_gain::{
    CANT_SPEAK_DISTANCE, CANT_SPEAK_GAIN, MAX_VEHICLE_LOSS, NO_VEHICLE_LOSS, SPEAKER_GAIN,
    UNDERWATER_LEVEL, speaker_audible_distance, vehicle_loss_gain, vehicle_volume_loss,
    volume_attenuation,
};
pub use crate::dsp_needs::{ensure_every_sender_dsp, ensure_sender_dsp};
pub use crate::placement::{HRTF_MAKEUP_GAIN_DB, hrtf_makeup_gain};
pub use crate::remote_tangent_sound::{
    RemoteSoundListener, RemoteSoundPlan, apply_stereo_mode, render_remote_tangent_sound,
};
pub use crate::voice_context::{VoiceFrameContext, VoiceOutcome};
pub use crate::voice_processor::{OWN_RADIO_VOICE_STREAM, VoiceProcessor};
pub use crate::voice_scratch::VoiceScratchCapacities;
