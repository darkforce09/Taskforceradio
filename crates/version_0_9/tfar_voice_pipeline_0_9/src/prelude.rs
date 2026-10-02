//! The names a caller of `tfar_voice_pipeline_0_9` imports with `use tfar_voice_pipeline_0_9::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
pub use crate::distance_gain::{radio_volume_multiplier, volume_from_distance};
pub use crate::own_voice::{OWN_VOICE_STREAM_NAME, append_own_voice};
pub use crate::voice_context::{OwnVoiceContext, VoiceFrameContext, VoiceOutcome};
pub use crate::voice_processor::VoiceProcessor;
pub use crate::voice_scratch::{ScratchCapacities, VoiceScratch};
