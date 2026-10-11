//! The owned interleaved two-channel buffer the voice pipelines process.
//!
//! **Role:** holds one block of interleaved `i16` stereo frames (left, right, left, right, ...) and
//! its frame count, reusing its allocation from block to block.
//!
//! **Position:** in `crates/audio`; filled by [`crate::sample_buffer::copy_to_stereo`] and
//! [`crate::sample_buffer::mono_to_stereo`] from TeamSpeak's sample arrays, processed by the filter, HRTF and
//! effect crates, and folded back by [`crate::sample_buffer::write_back_from_stereo`].
//!
//! **Signals and state:** owns its sample `Vec`; created once per client or radio and then used
//! from the TeamSpeak audio thread, which only reuses it.
//!
//! **Invariants:** the sample count is always exactly `2 * frames()`; changing the frame count
//! within [`StereoBuffer::capacity_frames`] never reallocates.

#[cfg(test)]
#[path = "tests/stereo_buffer.rs"]
mod tests;

/// An owned block of interleaved `i16` stereo frames: sample `2 * n` is the left channel of frame
/// `n`, sample `2 * n + 1` the right.
///
/// Create it once with [`StereoBuffer::with_frames`] (or reserve with
/// [`StereoBuffer::reserve_frames`]) for the largest block TeamSpeak delivers; after that, the
/// per-frame operations reuse the allocation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StereoBuffer {
    samples: Vec<i16>,
}

impl StereoBuffer {
    /// The channels in every frame: left and right.
    pub const CHANNELS: usize = 2;

    /// A silent buffer of `frames` frames. Allocates; call it outside the audio callbacks.
    pub fn with_frames(frames: usize) -> Self {
        Self {
            samples: vec![0; frames.saturating_mul(Self::CHANNELS)],
        }
    }

    /// The frames the buffer holds now.
    pub fn frames(&self) -> usize {
        self.samples.len() / Self::CHANNELS
    }

    /// The frames the buffer can hold without reallocating.
    pub fn capacity_frames(&self) -> usize {
        self.samples.capacity() / Self::CHANNELS
    }

    /// Makes room for at least `frames` frames in total, so later blocks up to that size never
    /// reallocate. Allocates when the capacity is smaller; call it outside the audio callbacks.
    pub fn reserve_frames(&mut self, frames: usize) {
        let wanted_samples = frames.saturating_mul(Self::CHANNELS);
        self.samples
            .reserve(wanted_samples.saturating_sub(self.samples.len()));
    }

    /// Sets the frame count to `frames`, keeping the samples of the frames that stay and filling
    /// new frames with silence. Reallocates only when `frames` exceeds
    /// [`StereoBuffer::capacity_frames`].
    pub fn set_frames(&mut self, frames: usize) {
        self.samples
            .resize(frames.saturating_mul(Self::CHANNELS), 0);
    }

    /// Silences every sample; the frame count stays.
    pub fn clear(&mut self) {
        self.samples.fill(0);
    }

    /// The interleaved samples, `2 * frames()` of them.
    pub fn as_slice(&self) -> &[i16] {
        &self.samples
    }

    /// The interleaved samples, mutably, `2 * frames()` of them.
    pub fn as_mut_slice(&mut self) -> &mut [i16] {
        &mut self.samples
    }

    /// Makes this buffer an exact copy of `other`, frame count included. Reallocates only when
    /// `other` holds more frames than [`StereoBuffer::capacity_frames`].
    pub fn copy_from(&mut self, other: &StereoBuffer) {
        self.samples.clear();
        self.samples.extend_from_slice(&other.samples);
    }

    /// Every frame as `[left, right]`. The sample count is always even, so no sample is left out.
    pub(crate) fn frame_pairs(&self) -> &[[i16; Self::CHANNELS]] {
        self.samples.as_chunks::<{ Self::CHANNELS }>().0
    }

    /// Every frame as a mutable `[left, right]`. The sample count is always even, so no sample is
    /// left out.
    pub(crate) fn frame_pairs_mut(&mut self) -> &mut [[i16; Self::CHANNELS]] {
        self.samples.as_chunks_mut::<{ Self::CHANNELS }>().0
    }
}
