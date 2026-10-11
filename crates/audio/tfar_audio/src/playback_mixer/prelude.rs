//! The names a caller of `tfar_audio::playback_mixer` imports with `use crate::playback_mixer::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::playback_mixer::mixer::{
    MAX_ACTIVE_PLAYBACKS, MAX_STREAMS, PLAYBACK_QUEUE_CAPACITY, PlaybackMixer,
};
pub use crate::playback_mixer::playback::{Playback, PlaybackId};
pub use crate::playback_mixer::playback_stream::STREAM_CAPACITY_FRAMES;
pub use crate::playback_mixer::stream_producer::StreamProducer;
pub use crate::playback_mixer::wave_library::WaveLibrary;
