//! The names a caller of `tfar_playback_mixer` imports with `use tfar_playback_mixer::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
pub use crate::playback::{Playback, PlaybackId};
pub use crate::playback_mixer::{
    MAX_ACTIVE_PLAYBACKS, MAX_STREAMS, PLAYBACK_QUEUE_CAPACITY, PlaybackMixer,
};
pub use crate::playback_stream::STREAM_CAPACITY_FRAMES;
pub use crate::stream_producer::StreamProducer;
pub use crate::wave_library::WaveLibrary;
