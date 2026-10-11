//! [`StreamProducer`]: the audio-thread end of a named stream, which appends stereo samples to an
//! `rtrb` ring the mixer drains, without waiting and without allocating.
//!
//! **Role:** the wait-free counterpart of `PlaybackMixer::append_stream`: one bounded ring of
//! interleaved stereo samples per stream, written by one producer (the captured-voice callback),
//! drained in order by `PlaybackMixer::mix_into` (the mixed-playback callback), keeping the newest
//! [`STREAM_CAPACITY_FRAMES`](crate::playback_mixer::STREAM_CAPACITY_FRAMES) frames.
//!
//! **Position:** a module of `tfar_audio::playback_mixer`; created by
//! `PlaybackMixer::stream_producer`, which keeps the reading side in its list of producer streams.
//!
//! **Signals and state:** the ring's producer, owned by the [`StreamProducer`]; the consumer behind
//! a `parking_lot` lock shared with the mixer, taken only with `try_lock`: by the mixer to drain,
//! and by the producer to drop the oldest frames when the ring is full.
//!
//! **Invariants:** `push` never blocks, never allocates and writes whole stereo frames only, so the
//! ring always holds whole frames; when the ring is full the oldest frames make room for the new
//! ones, unless the mixer is draining at that moment, when the frames that do not fit are dropped.

use std::sync::Arc;

use crate::sample_buffer::StereoBuffer;
use parking_lot::Mutex;
use rtrb::{Consumer, Producer, RingBuffer};

use crate::playback_mixer::output_block::mix_stereo_samples;
use crate::playback_mixer::playback_stream::STREAM_CAPACITY_FRAMES;

#[cfg(test)]
#[path = "tests/stream_producer.rs"]
mod tests;

/// The ring size of a producer stream in interleaved stereo samples: 500 ms at 48 kHz.
const PRODUCER_STREAM_CAPACITY_SAMPLES: usize = STREAM_CAPACITY_FRAMES * StereoBuffer::CHANNELS;

/// The reading side of one producer stream, shared by its [`StreamProducer`] and the mixer.
#[derive(Debug)]
pub(crate) struct ProducerStream {
    /// The stream's name.
    name: String,
    /// The ring's reading side; `try_lock` only on the audio threads.
    consumer: Mutex<Consumer<i16>>,
}

impl ProducerStream {
    /// The stream's name.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// The frames waiting to be mixed. Takes the lock; not for the audio threads.
    pub(crate) fn queued_frames(&self) -> usize {
        self.consumer.lock().slots() / StereoBuffer::CHANNELS
    }

    /// True once the [`StreamProducer`] is gone and every frame it pushed was mixed, so the mixer
    /// may forget the stream. Takes the lock; not for the audio threads.
    pub(crate) fn is_spent(&self) -> bool {
        let consumer = self.consumer.lock();
        consumer.is_abandoned() && consumer.is_empty()
    }

    /// Mixes the front of the ring into `block` of `channels` channels and removes what was mixed;
    /// returns whether any frame was mixed. Skips this block when the producer holds the lock to
    /// drop old frames. Never waits and never allocates.
    pub(crate) fn mix_front_into(&self, block: &mut [i16], channels: usize) -> bool {
        let Some(mut consumer) = self.consumer.try_lock() else {
            return false;
        };
        let readable = whole_frames(consumer.slots());
        let Ok(chunk) = consumer.read_chunk(readable) else {
            return false;
        };
        let (first, second) = chunk.as_slices();
        let mixed = mix_stereo_samples(block, channels, first.iter().chain(second).copied());
        let mixed_samples = mixed
            .saturating_mul(StereoBuffer::CHANNELS)
            .min(chunk.len());
        chunk.commit(mixed_samples);
        mixed > 0
    }
}

/// The writing end of a named stream that `PlaybackMixer::mix_into` mixes; see the module header.
///
/// Made by [`PlaybackMixer::stream_producer`](crate::playback_mixer::PlaybackMixer::stream_producer). One thread
/// pushes (the captured-voice callback), so [`push`](Self::push) takes `&mut self`. Dropping the
/// producer ends the stream once its queued frames are mixed.
#[derive(Debug)]
pub struct StreamProducer {
    /// The ring's writing side.
    producer: Producer<i16>,
    /// The reading side the mixer drains, shared for dropping the oldest frames.
    stream: Arc<ProducerStream>,
}

impl StreamProducer {
    /// A new stream called `name` with an empty ring of 500 ms, and the reading side the mixer
    /// keeps. Allocates; call it outside the audio callbacks.
    pub(crate) fn new(name: &str) -> (Self, Arc<ProducerStream>) {
        let (producer, consumer) = RingBuffer::new(PRODUCER_STREAM_CAPACITY_SAMPLES);
        let stream = Arc::new(ProducerStream {
            name: name.to_owned(),
            consumer: Mutex::new(consumer),
        });
        let producer = Self {
            producer,
            stream: Arc::clone(&stream),
        };
        (producer, stream)
    }

    /// The stream's name.
    pub fn name(&self) -> &str {
        self.stream.name()
    }

    /// Appends the whole stereo frames of the interleaved `stereo_samples` and returns how many
    /// samples went into the stream (always even; a trailing odd sample is ignored). Beyond 500 ms
    /// the oldest frames are dropped to make room, as with
    /// [`append_stream`](crate::playback_mixer::PlaybackMixer::append_stream); only while the mixer is draining
    /// this very stream are the newest frames that do not fit dropped instead. Never blocks and
    /// never allocates, so the captured-voice callback may call it.
    pub fn push(&mut self, stereo_samples: &[i16]) -> usize {
        let whole = whole_frames(stereo_samples.len());
        let newest_start = whole.saturating_sub(PRODUCER_STREAM_CAPACITY_SAMPLES);
        let incoming = stereo_samples.get(newest_start..whole).unwrap_or_default();
        let missing = incoming.len().saturating_sub(self.producer.slots());
        if missing > 0
            && let Some(mut consumer) = self.stream.consumer.try_lock()
        {
            // Both counts are whole frames, as every push and every read is.
            let dropped = missing.min(whole_frames(consumer.slots()));
            if let Ok(oldest) = consumer.read_chunk(dropped) {
                oldest.commit_all();
            }
        }
        let writable = whole_frames(self.producer.slots().min(incoming.len()));
        let kept_start = incoming.len().saturating_sub(writable);
        let kept = incoming.get(kept_start..).unwrap_or_default();
        match self.producer.push_entire_slice(kept) {
            Ok(()) => kept.len(),
            Err(_) => 0,
        }
    }
}

/// `samples` rounded down to whole stereo frames.
fn whole_frames(samples: usize) -> usize {
    samples - samples % StereoBuffer::CHANNELS
}
