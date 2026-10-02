//! A designed filter with one Direct Form II Transposed state per channel.
//!
//! **Role:** runs the sections of a [`SecondOrderSections`] cascade over interleaved frames in
//! place, in `f64`, keeping each channel's filter memory apart; [`MonoFilter`] is the one-channel
//! form the radio effects run sample by sample, and [`ChannelFilter::process_stereo_i16`] is the
//! stereo `i16` path of the original `processFilterStereo`
//! (`reference/tfar_1_0/ts/src/helpers.hpp`).
//!
//! **Position:** tier 1 of crates/audio; built from a [`FilterDesign`] by the client state and
//! radio effect crates, run by the voice pipelines on each TeamSpeak voice block.
//!
//! **Signals and state:** each filter owns its per-channel section states. Construction allocates
//! and designs the coefficients (about a millisecond for a Butterworth design), so it runs off
//! the audio thread; the processing methods and [`ChannelFilter::reset`] run on the TeamSpeak
//! audio thread and allocate nothing.
//!
//! **Invariants:** every channel runs the same sections with its own state, so channel `c` of the
//! output depends only on channel `c` of the input; processing never fails and never panics.

use biquad::{Biquad, Coefficients, DirectForm2Transposed};
use tfar_sample_buffer::{StereoBuffer, float_to_sample, sample_to_float};

use crate::error::Result;
use crate::filter_design::FilterDesign;
use crate::second_order_sections::{SecondOrderSection, SecondOrderSections};

#[cfg(test)]
#[path = "tests/channel_filter.rs"]
mod tests;

/// A one-channel filter: what the radio effects run on their mono signal.
pub type MonoFilter = ChannelFilter<1>;

/// A two-channel filter over interleaved left and right frames.
pub type StereoFilter = ChannelFilter<2>;

/// One section's Direct Form II Transposed state and coefficients, from `biquad`.
type SectionState = DirectForm2Transposed<f64>;

/// A designed filter with an independent Direct Form II Transposed state per channel, processing
/// in `f64`.
#[derive(Debug, Clone)]
pub struct ChannelFilter<const CHANNELS: usize> {
    sections: SecondOrderSections,
    channel_states: [Vec<SectionState>; CHANNELS],
}

impl<const CHANNELS: usize> ChannelFilter<CHANNELS> {
    /// Designs `design` for `sample_rate_hz` and builds a filter with silent state on every
    /// channel. Allocates and designs: call it off the audio thread.
    ///
    /// # Errors
    ///
    /// Every error of [`FilterDesign::second_order_sections`].
    pub fn new(design: FilterDesign, sample_rate_hz: f64) -> Result<Self> {
        Ok(Self::from_sections(
            design.second_order_sections(sample_rate_hz)?,
        ))
    }

    /// Builds a filter over already designed sections, with silent state on every channel.
    /// Allocates: call it off the audio thread.
    pub fn from_sections(sections: SecondOrderSections) -> Self {
        let states: Vec<SectionState> = sections
            .sections()
            .iter()
            .map(|section| DirectForm2Transposed::new(biquad_coefficients(section)))
            .collect();
        Self {
            channel_states: core::array::from_fn(|_| states.clone()),
            sections,
        }
    }

    /// The sections every channel runs.
    pub fn sections(&self) -> &SecondOrderSections {
        &self.sections
    }

    /// Silences the filter memory of every channel, as if no sample had been processed.
    pub fn reset(&mut self) {
        for state in self.channel_states.iter_mut().flatten() {
            state.reset_state();
        }
    }

    /// Filters one frame: sample `c` runs through channel `c`'s sections.
    pub fn process_frame(&mut self, frame: [f64; CHANNELS]) -> [f64; CHANNELS] {
        let mut output = frame;
        for (sample, states) in output.iter_mut().zip(self.channel_states.iter_mut()) {
            *sample = run_sections(states, *sample);
        }
        output
    }

    /// Filters interleaved `f64` frames in place.
    pub fn process_frames_f64(&mut self, frames: &mut [[f64; CHANNELS]]) {
        for frame in frames {
            *frame = self.process_frame(*frame);
        }
    }

    /// Filters interleaved `f32` frames in place; each sample is processed in `f64` and rounded
    /// back to `f32`.
    pub fn process_frames_f32(&mut self, frames: &mut [[f32; CHANNELS]]) {
        for frame in frames {
            for (sample, states) in frame.iter_mut().zip(self.channel_states.iter_mut()) {
                *sample = narrow_to_f32(run_sections(states, f64::from(*sample)));
            }
        }
    }
}

impl ChannelFilter<1> {
    /// Filters one mono sample.
    pub fn process_sample(&mut self, sample: f64) -> f64 {
        let [output] = self.process_frame([sample]);
        output
    }
}

impl ChannelFilter<2> {
    /// Filters a stereo `i16` block in place as the original `processFilterStereo` does: each
    /// sample becomes `sample / INPUT_SCALE`, runs through its channel's sections, is multiplied
    /// by `gain`, and is written back by [`clip_to_sample`]: above 1.0 `i16::MAX`, below -1.0
    /// `i16::MIN`, otherwise times `OUTPUT_SCALE` and truncated toward zero (`NaN` becomes 0).
    /// Allocates nothing.
    pub fn process_stereo_i16(&mut self, buffer: &mut StereoBuffer, gain: f32) {
        let gain = f64::from(gain);
        let (frames, _) = buffer.as_mut_slice().as_chunks_mut::<2>();
        for frame in frames {
            let input = frame.map(|sample| f64::from(sample_to_float(sample)));
            let output = self.process_frame(input);
            *frame = output.map(|value| clip_to_sample(narrow_to_f32(value * gain)));
        }
    }
}

/// The write-back of `processFilterStereo` (`reference/tfar_1_0/ts/src/helpers.hpp:185-187`): a
/// value above 1.0 is `i16::MAX` and one below -1.0 is `i16::MIN`; anything else goes through
/// [`float_to_sample`], times `OUTPUT_SCALE` and truncated toward zero, with `NaN` as 0.
fn clip_to_sample(value: f32) -> i16 {
    if value > 1.0 {
        i16::MAX
    } else if value < -1.0 {
        i16::MIN
    } else {
        float_to_sample(value)
    }
}

/// Runs `sample` through every section of one channel in order.
fn run_sections(states: &mut [SectionState], sample: f64) -> f64 {
    states
        .iter_mut()
        .fold(sample, |input, state| state.run(input))
}

/// The `biquad` coefficients of a normalised section.
fn biquad_coefficients(section: &SecondOrderSection) -> Coefficients<f64> {
    Coefficients {
        a1: section.a1,
        a2: section.a2,
        b0: section.b0,
        b1: section.b1,
        b2: section.b2,
    }
}

/// Rounds an `f64` sample to the nearest `f32`; values beyond the `f32` range become infinite,
/// which the `i16` conversion then clamps.
#[allow(
    clippy::cast_possible_truncation,
    reason = "narrowing a processed sample back to the f32 the caller holds is the intent"
)]
fn narrow_to_f32(value: f64) -> f32 {
    value as f32
}
