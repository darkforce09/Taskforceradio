//! The block adapter between TeamSpeak's variable voice frames and the fixed render block.
//!
//! **Role:** collects mono input samples into one fixed-size block and hands out the stereo
//! output of the block rendered before it, so any sequence of frame sizes (480, 960, 441, 1, ...)
//! becomes whole render blocks with a constant delay of exactly one block.
//!
//! **Position:** a module of `tfar_audio::hrtf`; owned by each [`crate::hrtf::HrtfSource`], which feeds it one
//! sample at a time from inside the TeamSpeak voice callbacks and renders full blocks through the
//! `hrtf` processor.
//!
//! **Signals and state:** two fixed buffers (the input block being filled, the output block being
//! drained) and one position shared by both; used from the TeamSpeak audio thread only. Nothing
//! allocates after `new`.
//!
//! **Invariants:** after `n` samples in, exactly `n` samples have come out; output sample `n` is
//! the rendering of input sample `n - block_frames`; the first `block_frames` outputs (after `new`
//! or `reset`) are silence; no input sample is dropped or repeated whatever the frame sizes.

#[cfg(test)]
#[path = "tests/block_adapter.rs"]
mod tests;

/// Renders one full block of mono input into stereo output of the same length.
pub(crate) trait BlockRenderer {
    /// Renders `input` into `output`, overwriting it; both hold one block.
    fn render_block(&mut self, input: &[f32], output: &mut [(f32, f32)]);
}

/// The double buffer that turns a stream of mono samples into whole render blocks.
///
/// The input block fills at the same position the output block drains, so when the input block
/// is full the output block is exactly used up: the renderer then overwrites the output block
/// from the input block and both start again at position 0.
#[derive(Debug)]
pub(crate) struct HrtfBlockAdapter {
    pending_input: Box<[f32]>,
    rendered_output: Box<[(f32, f32)]>,
    position: usize,
}

impl HrtfBlockAdapter {
    /// An adapter for blocks of `block_frames` frames, holding silence. Allocates both blocks.
    pub(crate) fn new(block_frames: usize) -> Self {
        Self {
            pending_input: vec![0.0; block_frames].into_boxed_slice(),
            rendered_output: vec![(0.0, 0.0); block_frames].into_boxed_slice(),
            position: 0,
        }
    }

    /// Takes one input sample and gives back the output sample one block older, rendering the
    /// next block through `renderer` when the input block is full.
    pub(crate) fn exchange(&mut self, input: f32, renderer: &mut impl BlockRenderer) -> (f32, f32) {
        let output = self
            .rendered_output
            .get(self.position)
            .copied()
            .unwrap_or((0.0, 0.0));
        if let Some(slot) = self.pending_input.get_mut(self.position) {
            *slot = input;
        }
        self.position = self.position.saturating_add(1);
        if self.position >= self.pending_input.len() {
            renderer.render_block(&self.pending_input, &mut self.rendered_output);
            self.position = 0;
        }
        output
    }

    /// Back to the state of `new`: both blocks silent, position 0. Allocates nothing.
    pub(crate) fn reset(&mut self) {
        self.pending_input.fill(0.0);
        self.rendered_output.fill((0.0, 0.0));
        self.position = 0;
    }
}
