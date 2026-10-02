//! Unit tests for the block adapter, with a renderer whose output is its input, so the delay and
//! the sample order are visible directly.

use super::*;

/// Renders each sample `x` as `(x, -x)` and counts the blocks it rendered.
#[derive(Default)]
struct MirrorRenderer {
    blocks_rendered: usize,
}

impl BlockRenderer for MirrorRenderer {
    fn render_block(&mut self, input: &[f32], output: &mut [(f32, f32)]) {
        assert_eq!(input.len(), output.len());
        for (sample, frame) in input.iter().zip(output.iter_mut()) {
            *frame = (*sample, -*sample);
        }
        self.blocks_rendered += 1;
    }
}

/// Feeds `total` samples numbered 1, 2, 3, ... in frames cycling through `frame_sizes` and returns
/// every output frame.
fn run(
    adapter: &mut HrtfBlockAdapter,
    renderer: &mut MirrorRenderer,
    frame_sizes: &[usize],
    total: usize,
) -> Vec<(f32, f32)> {
    let mut outputs = Vec::with_capacity(total);
    let mut next_sample = 1.0_f32;
    for frame_size in frame_sizes.iter().cycle() {
        if outputs.len() >= total {
            break;
        }
        let frame_size = (*frame_size).min(total - outputs.len());
        for _ in 0..frame_size {
            outputs.push(adapter.exchange(next_sample, renderer));
            next_sample += 1.0;
        }
    }
    outputs
}

#[test]
fn the_first_block_of_output_is_silence() {
    let mut adapter = HrtfBlockAdapter::new(8);
    let mut renderer = MirrorRenderer::default();
    let outputs = run(&mut adapter, &mut renderer, &[3], 8);
    assert!(outputs.iter().all(|frame| *frame == (0.0, 0.0)));
    assert_eq!(renderer.blocks_rendered, 1);
}

#[test]
fn every_sample_comes_out_exactly_one_block_later_whatever_the_frame_sizes() {
    let block_frames = 480;
    let mut adapter = HrtfBlockAdapter::new(block_frames);
    let mut renderer = MirrorRenderer::default();
    let total = 96_000;
    let outputs = run(&mut adapter, &mut renderer, &[480, 960, 441, 1], total);
    assert_eq!(outputs.len(), total);
    let mut delayed_sample = 0.0_f32;
    for (index, frame) in outputs.iter().enumerate() {
        let expected = if index < block_frames {
            0.0
        } else {
            delayed_sample += 1.0;
            delayed_sample
        };
        assert_eq!(*frame, (expected, -expected), "output frame {index}");
    }
    assert_eq!(renderer.blocks_rendered, total / block_frames);
}

#[test]
fn reset_brings_back_the_first_fill_silence() {
    let mut adapter = HrtfBlockAdapter::new(4);
    let mut renderer = MirrorRenderer::default();
    let _ = run(&mut adapter, &mut renderer, &[5], 10);
    adapter.reset();
    let outputs = run(&mut adapter, &mut renderer, &[2], 8);
    assert!(outputs[..4].iter().all(|frame| *frame == (0.0, 0.0)));
    assert_eq!(outputs[4], (1.0, -1.0));
}
