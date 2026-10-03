//! Loading and mixing of the radio sound WAV files and the own-voice stream into the outgoing
//! audio.
//!
//! **Role:** the radio sound player: loads the WAV files shipped in the mod's `radio-sounds` folder
//! through `hound`, refuses any format but 48 kHz 16-bit stereo, caches them, and mixes processed
//! playbacks and named streams into TeamSpeak's mixed-playback block.
//!
//! **Position:** `crates/audio`, tier 1, depending on `tfar_sample_buffer`; used by
//! `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** the WAV cache (`WaveLibrary`), and per mixer the queued and active
//! playbacks and the named streams. Playbacks are built and queued on non-audio threads, streams
//! are fed by the captured-voice callback, wait-free through a `StreamProducer` ring, and
//! `PlaybackMixer::mix_into` runs on the mixed-playback callback without waiting or allocating.
//!
//! **Invariants:** a playback's processing, filters included, runs and is freed before it is
//! queued; finished playbacks are freed off the audio thread; nothing is mixed past
//! `channels * frames` samples or into channels 2 and up.

mod crate_role;
mod error;
mod output_block;
mod playback;
mod playback_mixer;
mod playback_stream;
pub mod prelude;
mod stream_producer;
mod wave_library;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
pub use playback::{Playback, PlaybackId};
pub use playback_mixer::{
    MAX_ACTIVE_PLAYBACKS, MAX_STREAMS, PLAYBACK_QUEUE_CAPACITY, PlaybackMixer,
};
pub use playback_stream::STREAM_CAPACITY_FRAMES;
pub use stream_producer::StreamProducer;
pub use wave_library::WaveLibrary;
