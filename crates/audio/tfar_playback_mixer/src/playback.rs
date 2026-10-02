//! `Playback` and `PlaybackId`: one processed sound waiting to be mixed, and its name.
//!
//! **Role:** holds a sound's interleaved stereo samples, already processed, and the cursor of the
//! next frame to mix; names it by a process-wide counter.
//!
//! **Position:** a module of `tfar_playback_mixer`, tier 1 of crates/audio; built by the plugins
//! and voice pipelines from a `WaveLibrary` sound and handed to `PlaybackMixer::add`. The
//! specification is `playbackWavStereo` and `playbackWavProcessing` of
//! `reference/tfar_1_0/ts/src/PlaybackHandler.cpp`, without their processing thread and without
//! the `fileName + rand()` key (F-005).
//!
//! **Signals and state:** a playback is built on any thread but the audio thread: `Playback::new`
//! copies the samples and runs the caller's processing there, so the gain, filters and panning
//! closure is consumed and its filters freed before the playback reaches the mixer. The audio
//! thread then only advances the cursor. The id counter is a process-wide atomic.
//!
//! **Invariants:** the samples are whole stereo frames; the cursor never passes the last frame; ids
//! are unique within the process and never reused.

use core::fmt;
use core::sync::atomic::{AtomicU64, Ordering};

use tfar_sample_buffer::StereoBuffer;

use crate::error::{Error, Result};
use crate::output_block::mix_stereo_samples;

#[cfg(test)]
#[path = "tests/playback.rs"]
mod tests;

/// The id the next playback gets; ids start at 1.
static NEXT_PLAYBACK_ID: AtomicU64 = AtomicU64::new(1);

/// The name of one playback: a number from a process-wide counter, unique and never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlaybackId(u64);

impl PlaybackId {
    /// A fresh id, one above the last one handed out.
    pub fn next() -> Self {
        Self(NEXT_PLAYBACK_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// The number behind the id.
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for PlaybackId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "playback {}", self.0)
    }
}

/// One processed sound: its interleaved stereo samples and the cursor of the next frame to mix.
#[derive(Debug)]
pub struct Playback {
    id: PlaybackId,
    samples: StereoBuffer,
    next_frame: usize,
}

impl Playback {
    /// A playback of the interleaved stereo `stereo_samples`, processed by `process`.
    ///
    /// The samples are copied into an owned [`StereoBuffer`] and `process` runs on it right here,
    /// on the calling thread: gain, filters and panning happen once, off the audio thread, and the
    /// closure (with any filter it owns) is dropped before this returns. `process` may change the
    /// frame count. Allocates; call it outside the audio callbacks.
    ///
    /// # Errors
    ///
    /// [`Error::OddStereoSampleCount`] when `stereo_samples` is not whole stereo frames.
    pub fn new(stereo_samples: &[i16], process: impl FnOnce(&mut StereoBuffer)) -> Result<Self> {
        if !stereo_samples.len().is_multiple_of(StereoBuffer::CHANNELS) {
            return Err(Error::OddStereoSampleCount {
                samples: stereo_samples.len(),
            });
        }
        let mut samples = StereoBuffer::with_frames(stereo_samples.len() / StereoBuffer::CHANNELS);
        for (target, source) in samples.as_mut_slice().iter_mut().zip(stereo_samples) {
            *target = *source;
        }
        process(&mut samples);
        Ok(Self::from_stereo(samples))
    }

    /// A playback of `samples` as they are, already processed by the caller.
    pub fn from_stereo(samples: StereoBuffer) -> Self {
        Self {
            id: PlaybackId::next(),
            samples,
            next_frame: 0,
        }
    }

    /// The playback's id.
    pub const fn id(&self) -> PlaybackId {
        self.id
    }

    /// The frames the playback holds in total.
    pub fn frames(&self) -> usize {
        self.samples.frames()
    }

    /// The frames not yet mixed.
    pub fn remaining_frames(&self) -> usize {
        self.frames().saturating_sub(self.next_frame)
    }

    /// Whether every frame has been mixed.
    pub fn is_finished(&self) -> bool {
        self.remaining_frames() == 0
    }

    /// The processed samples, interleaved stereo.
    pub fn samples(&self) -> &[i16] {
        self.samples.as_slice()
    }

    /// Adds the next frames into `block` of `channels` channels and advances the cursor past them;
    /// returns whether any frame was mixed. Called on the audio thread; nothing allocates.
    pub(crate) fn mix_next_into(&mut self, block: &mut [i16], channels: usize) -> bool {
        let start = self.next_frame.saturating_mul(StereoBuffer::CHANNELS);
        let remaining = self.samples.as_slice().get(start..).unwrap_or_default();
        let mixed = mix_stereo_samples(block, channels, remaining.iter().copied());
        self.next_frame = self.next_frame.saturating_add(mixed).min(self.frames());
        mixed > 0
    }
}
