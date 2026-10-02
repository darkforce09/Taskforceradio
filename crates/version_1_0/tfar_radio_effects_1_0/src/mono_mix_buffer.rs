//! `MonoMixBuffer`: the reusable mono block [`crate::process_radio_effect`] mixes a stereo block
//! into before the effect runs.

#[cfg(test)]
#[path = "tests/mono_mix_buffer.rs"]
mod tests;

/// The mono `f32` scratch block of [`crate::process_radio_effect`]. Create it once per voice
/// pipeline for the largest block TeamSpeak delivers; after that, mixing reuses its allocation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MonoMixBuffer {
    samples: Vec<f32>,
}

impl MonoMixBuffer {
    /// A buffer with room for `frames` mono samples. Allocates: call it off the audio thread.
    pub fn with_frames(frames: usize) -> Self {
        Self {
            samples: Vec::with_capacity(frames),
        }
    }

    /// The frames the buffer holds without reallocating.
    pub fn capacity_frames(&self) -> usize {
        self.samples.capacity()
    }

    /// The block for `frames` mono samples, its old contents left in place and new samples
    /// silent. Reallocates only when `frames` exceeds [`MonoMixBuffer::capacity_frames`].
    pub(crate) fn block(&mut self, frames: usize) -> &mut [f32] {
        self.samples.resize(frames, 0.0);
        &mut self.samples
    }
}
