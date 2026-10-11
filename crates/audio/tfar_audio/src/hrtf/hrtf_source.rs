//! `HrtfSource`: one sound source rendered binaurally, frame by frame.
//!
//! **Role:** keeps everything one source needs between TeamSpeak voice frames (its copy of the
//! prepared sphere, the convolution tails of both ears, the previous direction and distance gain)
//! and feeds TeamSpeak's variable frames through the block adapter into the `hrtf` processor.
//!
//! **Position:** a module of `tfar_audio::hrtf`; created from an [`HrirLibrary`] by the voice pipelines,
//! one per (speaker, path) as the audio pipeline document describes, and called from the TeamSpeak
//! voice callbacks.
//!
//! **Signals and state:** per-source state only, used from the TeamSpeak audio thread. `new`
//! allocates (a copy of the prepared sphere, about two megabytes, and the pre-sized tails and
//! blocks); `process`, `process_mono` and `reset` allocate nothing, except that `output` grows
//! when a frame is longer than any before it. `process` and `process_mono` read the monotonic
//! clock once per frame for the idle restart.
//!
//! **Invariants:** the output lags the input by exactly [`ADDED_LATENCY_FRAMES`]; zero input gives
//! zero output; a frame arriving more than [`IDLE_RESET_AFTER`] after the previous one starts
//! from silence, as after `reset`; the processor is only ever called with whole render blocks, so
//! the `hrtf` crate's length assertions always hold; between two blocks the direction swings by
//! less than about 154° (a reversal snaps instead), so the crate's interpolated sampling vector
//! never passes near the sphere's centre, where it would find no triangle.

use std::time::Instant;

use crate::sample_buffer::{StereoBuffer, float_to_sample, sample_to_float};
use hrtf::{HrtfContext, HrtfProcessor};

use crate::hrtf::block_adapter::{BlockRenderer, HrtfBlockAdapter};
use crate::hrtf::hrir_library::HrirLibrary;
use crate::hrtf::listener_relative_direction::ListenerRelativeDirection;
#[cfg(doc)]
use crate::hrtf::render_timing::ADDED_LATENCY_FRAMES;
use crate::hrtf::render_timing::{IDLE_RESET_AFTER, RENDER_BLOCK_FRAMES};

#[cfg(test)]
#[path = "tests/hrtf_source.rs"]
mod tests;

/// The cosine below which a change of direction between two blocks counts as a reversal: about
/// 154°. Interpolating across a reversal would pass near the sphere's centre, so the new block
/// starts at the new direction instead.
const REVERSAL_COSINE: f32 = -0.9;

/// One sound source rendered through the HRIR sphere: mono in, binaural stereo out.
///
/// Create one per source with [`HrtfSource::new`] outside the audio callbacks if possible, and
/// keep it for as long as the source sounds; call [`HrtfSource::reset`] when the source jumps
/// (a new speaker on the same slot, a teleport), so nothing of the old sound rings on. A source
/// fed nothing for longer than [`IDLE_RESET_AFTER`] resets itself on its next frame, so the end
/// of one utterance never replays at the start of the next.
#[derive(Debug)]
pub struct HrtfSource {
    adapter: HrtfBlockAdapter,
    renderer: SourceRenderer,
    last_input: Option<Instant>,
}

impl HrtfSource {
    /// A silent source rendering with `library`'s sphere. Allocates a copy of the prepared sphere
    /// (about two megabytes for the IRCAM spheres) and every buffer the source will use.
    pub fn new(library: &HrirLibrary) -> Self {
        let tail_length = library.hrir_length().saturating_sub(1);
        Self {
            adapter: HrtfBlockAdapter::new(RENDER_BLOCK_FRAMES),
            renderer: SourceRenderer {
                processor: library.new_processor(),
                previous_left_tail: vec![0.0; tail_length],
                previous_right_tail: vec![0.0; tail_length],
                previous_sampling_vector: None,
                previous_distance_gain: None,
                target_sampling_vector: ListenerRelativeDirection::STRAIGHT_AHEAD
                    .sphere_sampling_vector(),
                target_distance_gain: 0.0,
            },
            last_input: None,
        }
    }

    /// Renders the stereo frame `input` from `direction` at `distance_gain` into `output`, which
    /// is set to `input.frames()` frames and overwritten.
    ///
    /// The input is down-mixed to mono as `(left + right) / 2`. The original TFAR plugins fed
    /// only the left channel to their direction stage (and the `hrtf` crate's own interleaved
    /// input reads only the left sample); the voice reaching this stage is mono duplicated into
    /// both channels, where the mean equals the left channel, and where the two channels differ
    /// the mean keeps both instead of dropping one.
    ///
    /// The direction and gain apply from the next render block on; the `hrtf` processor glides
    /// to them across the block's interpolation steps. A non-finite or negative gain is silence.
    /// The output lags the input by [`ADDED_LATENCY_FRAMES`].
    pub fn process(
        &mut self,
        input: &StereoBuffer,
        output: &mut StereoBuffer,
        direction: ListenerRelativeDirection,
        distance_gain: f32,
    ) {
        self.restart_after_idle(Instant::now());
        self.renderer.set_target(direction, distance_gain);
        output.set_frames(input.frames());
        let input_frames = input.as_slice().as_chunks::<{ StereoBuffer::CHANNELS }>().0;
        let output_frames = output
            .as_mut_slice()
            .as_chunks_mut::<{ StereoBuffer::CHANNELS }>()
            .0;
        for ([left_in, right_in], frame_out) in input_frames.iter().zip(output_frames) {
            let mono = f32::midpoint(sample_to_float(*left_in), sample_to_float(*right_in));
            *frame_out = self.exchange(mono);
        }
    }

    /// Renders the mono samples `input` from `direction` at `distance_gain` into `output`, which
    /// is set to `input.len()` frames and overwritten; otherwise as [`HrtfSource::process`].
    pub fn process_mono(
        &mut self,
        input: &[i16],
        output: &mut StereoBuffer,
        direction: ListenerRelativeDirection,
        distance_gain: f32,
    ) {
        self.restart_after_idle(Instant::now());
        self.renderer.set_target(direction, distance_gain);
        output.set_frames(input.len());
        let output_frames = output
            .as_mut_slice()
            .as_chunks_mut::<{ StereoBuffer::CHANNELS }>()
            .0;
        for (sample, frame_out) in input.iter().zip(output_frames) {
            *frame_out = self.exchange(sample_to_float(*sample));
        }
    }

    /// Forgets the source's past: the pending block, the rendered block, both convolution tails
    /// and the previous direction and gain. The next output is the first-fill silence again, and
    /// the next block starts at its direction without gliding. Allocates nothing.
    pub fn reset(&mut self) {
        self.adapter.reset();
        self.renderer.reset();
        self.last_input = None;
    }

    /// Resets the source when its last input lies more than [`IDLE_RESET_AFTER`] before `now`,
    /// then stamps `now` as its last input. Takes the time as `now`, so tests choose it.
    fn restart_after_idle(&mut self, now: Instant) {
        if self
            .last_input
            .is_some_and(|last| now.saturating_duration_since(last) > IDLE_RESET_AFTER)
        {
            self.adapter.reset();
            self.renderer.reset();
        }
        self.last_input = Some(now);
    }

    /// One mono sample in, one stereo frame out (one block older).
    fn exchange(&mut self, mono: f32) -> [i16; StereoBuffer::CHANNELS] {
        let (left, right) = self.adapter.exchange(mono, &mut self.renderer);
        [float_to_sample(left), float_to_sample(right)]
    }
}

/// The per-source state the `hrtf` processor needs between render blocks.
struct SourceRenderer {
    processor: HrtfProcessor,
    previous_left_tail: Vec<f32>,
    previous_right_tail: Vec<f32>,
    previous_sampling_vector: Option<hrtf::Vec3>,
    previous_distance_gain: Option<f32>,
    target_sampling_vector: hrtf::Vec3,
    target_distance_gain: f32,
}

impl SourceRenderer {
    /// Sets the direction and gain the next block glides to.
    fn set_target(&mut self, direction: ListenerRelativeDirection, distance_gain: f32) {
        self.target_sampling_vector = direction.sphere_sampling_vector();
        self.target_distance_gain = if distance_gain.is_finite() && distance_gain > 0.0 {
            distance_gain
        } else {
            0.0
        };
    }

    /// Clears the tails and forgets the previous direction and gain.
    fn reset(&mut self) {
        self.previous_left_tail.fill(0.0);
        self.previous_right_tail.fill(0.0);
        self.previous_sampling_vector = None;
        self.previous_distance_gain = None;
    }
}

impl BlockRenderer for SourceRenderer {
    fn render_block(&mut self, input: &[f32], output: &mut [(f32, f32)]) {
        output.fill((0.0, 0.0));
        if input.len() != RENDER_BLOCK_FRAMES || output.len() < RENDER_BLOCK_FRAMES {
            return;
        }
        let new_vector = self.target_sampling_vector;
        let previous_vector = block_start_vector(self.previous_sampling_vector, new_vector);
        let new_gain = self.target_distance_gain;
        let previous_gain = self.previous_distance_gain.unwrap_or(new_gain);
        self.processor.process_samples(HrtfContext {
            source: input,
            output,
            new_sample_vector: new_vector,
            prev_sample_vector: previous_vector,
            prev_left_samples: &mut self.previous_left_tail,
            prev_right_samples: &mut self.previous_right_tail,
            new_distance_gain: new_gain,
            prev_distance_gain: previous_gain,
        });
        self.previous_sampling_vector = Some(new_vector);
        self.previous_distance_gain = Some(new_gain);
    }
}

impl core::fmt::Debug for SourceRenderer {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SourceRenderer")
            .field("previous_sampling_vector", &self.previous_sampling_vector)
            .field("previous_distance_gain", &self.previous_distance_gain)
            .field("target_sampling_vector", &self.target_sampling_vector)
            .field("target_distance_gain", &self.target_distance_gain)
            .finish_non_exhaustive()
    }
}

/// The sampling vector a block glides from to `new_vector`: the previous block's vector, or
/// `new_vector` itself (no glide) for the first block, after a reset, and across a reversal.
fn block_start_vector(previous: Option<hrtf::Vec3>, new_vector: hrtf::Vec3) -> hrtf::Vec3 {
    match previous {
        Some(previous) if cosine_between(previous, new_vector) >= REVERSAL_COSINE => previous,
        _ => new_vector,
    }
}

/// The cosine of the angle between two unit vectors.
fn cosine_between(first: hrtf::Vec3, second: hrtf::Vec3) -> f32 {
    first.x * second.x + first.y * second.y + first.z * second.z
}
