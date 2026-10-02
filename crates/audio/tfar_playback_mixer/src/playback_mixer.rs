//! `PlaybackMixer`: the playbacks and streams added into TeamSpeak's mixed-playback block.
//!
//! **Role:** takes processed playbacks from any thread and named stream samples from the
//! captured-voice callback, and adds them into channels 0 and 1 of each mixed-playback block,
//! saturating, with TeamSpeak's channel fill mask.
//!
//! **Position:** a module of `tfar_playback_mixer`, tier 1 of crates/audio; one mixer per server
//! connection, held by the plugin apps and fed by the voice pipelines. The specification is
//! `PlaybackHandler::onEditMixedPlaybackVoiceDataEvent` in
//! `reference/tfar_1_0/ts/src/PlaybackHandler.cpp:36-79` and
//! `ts3plugin_onEditMixedPlaybackVoiceDataEvent` in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:2600-2651`.
//!
//! **Signals and state:** three locks. The hand-off lock is taken briefly by `add` and
//! `release_finished` on non-audio threads; it guards the producer of the `rtrb` queue of new
//! playbacks and the consumer of the queue of finished ones, which `add` drops there, off the
//! audio thread. The mixing lock guards the audio thread's side (the other queue ends and the
//! active playbacks); `mix_into` takes it with `try_lock`, so it never waits. The stream lock
//! guards the named streams: `append_stream` (captured-voice thread) holds it for one block's
//! copy, and `mix_into` takes it with `try_lock` and leaves the streams queued on contention.
//!
//! **Invariants:** `mix_into` never allocates and never touches samples past `channels * frames`
//! or channels 2 and up; at most [`MAX_ACTIVE_PLAYBACKS`] playbacks are active; a finished
//! playback is removed in the block it finishes in.

use parking_lot::Mutex;
use rtrb::{Consumer, Producer, PushError, RingBuffer};
use tfar_sample_buffer::StereoBuffer;

use crate::error::{Error, Result};
use crate::output_block::{checked_block, silence_unfilled_channels, written_channel_bits};
use crate::playback::{Playback, PlaybackId};
use crate::playback_stream::PlaybackStream;

#[cfg(test)]
#[path = "tests/playback_mixer.rs"]
mod tests;

/// The most playbacks mixed at once; more wait in the queue until a slot frees.
pub const MAX_ACTIVE_PLAYBACKS: usize = 64;
/// The most playbacks waiting for the audio thread; `add` refuses beyond.
pub const PLAYBACK_QUEUE_CAPACITY: usize = 64;
/// The most named streams a mixer holds.
pub const MAX_STREAMS: usize = 8;
/// The finished playbacks the queue back from the audio thread holds before the audio thread has
/// to drop one itself: every active and queued playback can finish before the next `add`.
const FINISHED_QUEUE_CAPACITY: usize = MAX_ACTIVE_PLAYBACKS + PLAYBACK_QUEUE_CAPACITY;

/// The non-audio threads' ends of the two queues.
struct HandOff {
    new_playbacks: Producer<Playback>,
    finished_playbacks: Consumer<Playback>,
}

/// The audio thread's ends of the two queues and the playbacks it is mixing.
struct MixingState {
    new_playbacks: Consumer<Playback>,
    finished_playbacks: Producer<Playback>,
    active: Vec<Playback>,
}

/// Mixes processed playbacks and named streams into TeamSpeak's mixed-playback blocks.
pub struct PlaybackMixer {
    hand_off: Mutex<HandOff>,
    mixing: Mutex<MixingState>,
    streams: Mutex<Vec<PlaybackStream>>,
}

impl Default for PlaybackMixer {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for PlaybackMixer {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PlaybackMixer")
            .finish_non_exhaustive()
    }
}

impl PlaybackMixer {
    /// An empty mixer with its queues and lists allocated at full size. Call it outside the audio
    /// callbacks.
    pub fn new() -> Self {
        let (new_producer, new_consumer) = RingBuffer::new(PLAYBACK_QUEUE_CAPACITY);
        let (finished_producer, finished_consumer) = RingBuffer::new(FINISHED_QUEUE_CAPACITY);
        Self {
            hand_off: Mutex::new(HandOff {
                new_playbacks: new_producer,
                finished_playbacks: finished_consumer,
            }),
            mixing: Mutex::new(MixingState {
                new_playbacks: new_consumer,
                finished_playbacks: finished_producer,
                active: Vec::with_capacity(MAX_ACTIVE_PLAYBACKS),
            }),
            streams: Mutex::new(Vec::with_capacity(MAX_STREAMS)),
        }
    }

    /// Queues `playback` for the audio thread and returns its id; first drops the playbacks that
    /// have finished since the last call. Callable from any thread but the audio thread.
    ///
    /// # Errors
    ///
    /// [`Error::PlaybackQueueFull`] when [`PLAYBACK_QUEUE_CAPACITY`] playbacks are already waiting;
    /// the playback is dropped.
    pub fn add(&self, playback: Playback) -> Result<PlaybackId> {
        let id = playback.id();
        let mut hand_off = self.hand_off.lock();
        while hand_off.finished_playbacks.pop().is_ok() {}
        hand_off
            .new_playbacks
            .push(playback)
            .map_err(|PushError::Full(_)| Error::PlaybackQueueFull { id })?;
        Ok(id)
    }

    /// Drops the playbacks that have finished since the last call, off the audio thread; returns
    /// how many. The plugin's service tick calls it so finished sounds are freed without an `add`.
    pub fn release_finished(&self) -> usize {
        let mut hand_off = self.hand_off.lock();
        let mut released = 0_usize;
        while hand_off.finished_playbacks.pop().is_ok() {
            released = released.saturating_add(1);
        }
        released
    }

    /// Appends interleaved stereo samples to the stream called `name`, creating it on first use;
    /// beyond 500 ms the oldest frames are dropped. The first append for a name allocates the
    /// stream once; later ones never allocate.
    ///
    /// # Errors
    ///
    /// [`Error::OddStereoSampleCount`] when `stereo_samples` is not whole frames, and
    /// [`Error::TooManyStreams`] when `name` is new and [`MAX_STREAMS`] streams exist.
    pub fn append_stream(&self, name: &str, stereo_samples: &[i16]) -> Result<()> {
        if !stereo_samples.len().is_multiple_of(StereoBuffer::CHANNELS) {
            return Err(Error::OddStereoSampleCount {
                samples: stereo_samples.len(),
            });
        }
        let mut streams = self.streams.lock();
        if let Some(stream) = streams.iter_mut().find(|stream| stream.name() == name) {
            stream.append(stereo_samples);
            return Ok(());
        }
        if streams.len() >= MAX_STREAMS {
            return Err(Error::TooManyStreams {
                name: name.to_owned(),
                maximum: MAX_STREAMS,
            });
        }
        let mut stream = PlaybackStream::new(name);
        stream.append(stereo_samples);
        streams.push(stream);
        Ok(())
    }

    /// Adds every active playback and stream into the first `frames` frames of the interleaved
    /// `samples`, `channels` per frame: the mixed-playback callback.
    ///
    /// Channel 0 and channel 1 (channel 0 alone in a mono block) are zeroed first where their bit
    /// in `channel_fill_mask` is clear; then each playback and stream is added into them,
    /// saturating, and channels 2 and up are left as they are. When anything was mixed, the bits
    /// of the written channels are set in `channel_fill_mask`. Playbacks that finish are removed.
    /// When another thread holds a lock, the playbacks or streams wait for the next block; this
    /// never blocks and never allocates.
    ///
    /// # Errors
    ///
    /// [`Error::NoChannels`], [`Error::SampleCountOverflow`] or [`Error::SampleSliceTooShort`] when
    /// the counts do not describe `samples`; nothing is touched then.
    pub fn mix_into(
        &self,
        samples: &mut [i16],
        channels: usize,
        frames: usize,
        channel_fill_mask: &mut u32,
    ) -> Result<()> {
        let block = checked_block(samples, channels, frames)?;
        silence_unfilled_channels(block, channels, *channel_fill_mask);
        let mut wrote = false;
        if let Some(mut mixing) = self.mixing.try_lock() {
            wrote |= mix_playbacks(&mut mixing, block, channels);
        }
        if let Some(mut streams) = self.streams.try_lock() {
            for stream in streams.iter_mut() {
                wrote |= stream.mix_front_into(block, channels);
            }
        }
        if wrote {
            *channel_fill_mask |= written_channel_bits(channels);
        }
        Ok(())
    }

    /// The playbacks being mixed now, not counting those still queued. Takes the mixing lock;
    /// not for the audio thread.
    pub fn active_playback_count(&self) -> usize {
        self.mixing.lock().active.len()
    }

    /// The frames queued in the stream called `name`, or `None` when there is no such stream.
    pub fn stream_queued_frames(&self, name: &str) -> Option<usize> {
        self.streams
            .lock()
            .iter()
            .find(|stream| stream.name() == name)
            .map(PlaybackStream::queued_frames)
    }
}

/// Moves queued playbacks into the free active slots, mixes every active playback into `block`
/// and hands the finished ones back; returns whether any frame was mixed.
fn mix_playbacks(mixing: &mut MixingState, block: &mut [i16], channels: usize) -> bool {
    let MixingState {
        new_playbacks,
        finished_playbacks,
        active,
    } = mixing;
    while active.len() < MAX_ACTIVE_PLAYBACKS {
        let Ok(playback) = new_playbacks.pop() else {
            break;
        };
        active.push(playback);
    }
    let mut wrote = false;
    for playback in active.iter_mut() {
        wrote |= playback.mix_next_into(block, channels);
    }
    for finished in active.extract_if(.., |playback| playback.is_finished()) {
        // A full queue back means nobody has called `add` or `release_finished` for a long
        // time; the playback is then dropped here, which frees memory but never allocates.
        if let Err(PushError::Full(dropped)) = finished_playbacks.push(finished) {
            drop(dropped);
        }
    }
    wrote
}
