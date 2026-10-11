//! `PlaybackStream`: a named, bounded FIFO of stereo samples that one callback appends to and the
//! mixer drains, such as the `my_radio_voice` stream of the player's own radio voice.
//!
//! **Role:** keeps the newest [`STREAM_CAPACITY_FRAMES`] frames of a continuous stream, dropping
//! the oldest beyond that, and mixes its front into a playback block.
//!
//! **Position:** a module of `tfar_audio::playback_mixer`; owned by
//! `PlaybackMixer`. It replaces the `my_radio_voice` raw playback of
//! `reference/tfar_1_0/ts/src/PlaybackHandler.cpp` (`playbackWavRaw`, appended from
//! `reference/tfar_1_0/ts/src/plugin.cpp:818`) and the deque of
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:539-564`, both of which grew without bound.
//!
//! **Signals and state:** appended by the captured-voice callback and drained by the
//! mixed-playback callback, two TeamSpeak audio threads, under the mixer's stream lock. The
//! sample queue is allocated once, at its full capacity, when the stream is created.
//!
//! **Invariants:** the queue holds whole stereo frames and never more than
//! [`STREAM_CAPACITY_FRAMES`] of them, so it never reallocates after creation.

use std::collections::VecDeque;

use crate::sample_buffer::StereoBuffer;

use crate::playback_mixer::output_block::mix_stereo_samples;

#[cfg(test)]
#[path = "tests/playback_stream.rs"]
mod tests;

/// The most frames a stream holds: 500 ms at 48 kHz.
///
/// The captured-voice and mixed-playback callbacks run at the same rate, so in steady state a
/// stream holds about one block. When the playback side stalls or stops while capture goes on,
/// the original queues grew without limit, in memory and in delay. Dropping the oldest frames
/// keeps the newest: once playback resumes, the sidetone lags by at most half a second instead
/// of replaying stale speech, and half a second still covers many 10 to 20 ms blocks of jitter
/// between the two callbacks.
pub const STREAM_CAPACITY_FRAMES: usize = 24_000;

/// [`STREAM_CAPACITY_FRAMES`] as interleaved stereo samples.
const STREAM_CAPACITY_SAMPLES: usize = STREAM_CAPACITY_FRAMES * StereoBuffer::CHANNELS;

/// A named FIFO of interleaved stereo samples, bounded to [`STREAM_CAPACITY_FRAMES`] frames.
#[derive(Debug)]
pub(crate) struct PlaybackStream {
    name: String,
    samples: VecDeque<i16>,
}

impl PlaybackStream {
    /// An empty stream called `name`, its queue allocated at full capacity.
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            samples: VecDeque::with_capacity(STREAM_CAPACITY_SAMPLES),
        }
    }

    /// The stream's name.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// The frames waiting to be mixed.
    pub(crate) fn queued_frames(&self) -> usize {
        self.samples.len() / StereoBuffer::CHANNELS
    }

    /// Appends whole stereo frames, dropping the oldest frames beyond the capacity. The caller has
    /// checked that `stereo_samples` is whole frames. Never reallocates.
    pub(crate) fn append(&mut self, stereo_samples: &[i16]) {
        let newest = stereo_samples.len().saturating_sub(STREAM_CAPACITY_SAMPLES);
        let kept_input = stereo_samples.get(newest..).unwrap_or_default();
        let overflow = self
            .samples
            .len()
            .saturating_add(kept_input.len())
            .saturating_sub(STREAM_CAPACITY_SAMPLES);
        self.samples.drain(..overflow.min(self.samples.len()));
        self.samples.extend(kept_input.iter().copied());
    }

    /// Mixes the front of the queue into `block` of `channels` channels and removes what was
    /// mixed; returns whether any frame was mixed. Nothing allocates.
    pub(crate) fn mix_front_into(&mut self, block: &mut [i16], channels: usize) -> bool {
        let mixed = mix_stereo_samples(block, channels, self.samples.iter().copied());
        let consumed = mixed
            .saturating_mul(StereoBuffer::CHANNELS)
            .min(self.samples.len());
        self.samples.drain(..consumed);
        mixed > 0
    }
}
