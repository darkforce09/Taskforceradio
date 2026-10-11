//! `Placement`: where a sound comes from and how loud, rendered by HRTF or the simple ILD.
//!
//! **Role:** carry the direction and level of one sound (direct speech or a speaker radio) from
//! planning to the audio path, and render it: through the sound's `HrtfSource` when it has one,
//! through the 1.0 simple ILD (`reference/tfar_1_0/ts/src/helpers.cpp:12-23`) when it has none.
//!
//! **Position:** a module of `tfar_plugin_logic_1_0::voice_pipeline`; built by the direct-speech and radio
//! planning, rendered by their audio-path halves.
//!
//! **Signals and state:** none of its own; the HRTF source it renders through belongs to the
//! sender's `ClientEffects`. Runs on the TeamSpeak audio thread.
//!
//! **Invariants:** one panner per sound (HRTF or ILD, never both); rendering allocates nothing
//! once the scratch buffer holds the frame.

use tfar_audio::hrtf::{HrtfSource, ListenerRelativeDirection};
use tfar_audio::sample_buffer::{StereoBuffer, apply_stereo_gain};
use tfar_audio::spatial_audio::{ILD_COEFFICIENT_1_0, stereo_pan_gains};
use tfar_foundation::geometry::{Direction, Position};

use crate::voice_pipeline::distance_gain::gain_as_f32;

#[cfg(test)]
#[path = "tests/placement.rs"]
mod tests;

/// The fixed gain after the HRTF on direct speech, in decibels.
///
/// The compensated IRCAM sphere is diffuse-field equalised, so a source straight ahead leaves
/// the HRTF about 4 dB below its input (measured in `tfar_audio::hrtf`). The original's X3DAudio
/// panning did not lose that level, so direct speech gets +4 dB after the HRTF and straight
/// ahead keeps its original loudness. Speaker radios get no makeup: the original's simple ILD
/// they replace is itself 0.625 (−4.1 dB) straight ahead, about what the HRTF leaves.
pub const HRTF_MAKEUP_GAIN_DB: f64 = 4.0;

/// [`HRTF_MAKEUP_GAIN_DB`] as a linear factor.
pub fn hrtf_makeup_gain() -> f64 {
    10.0_f64.powf(HRTF_MAKEUP_GAIN_DB / 20.0)
}

/// Where a sound is heard from and at what level, for both renderers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Placement {
    /// The direction from the listener's head to the source.
    pub(crate) direction: ListenerRelativeDirection,
    /// The level the HRTF renders at.
    pub(crate) hrtf_gain: f32,
    /// The (left, right) gains of the simple-ILD fallback, level included.
    pub(crate) fallback_gains: (f32, f32),
}

impl Placement {
    /// The placement of a source at `source` for a listener at `listener` looking along
    /// `listener_view`, at `level`; the HRTF level is `level · hrtf_makeup`.
    pub(crate) fn new(
        listener: Position,
        listener_view: Direction,
        source: Position,
        level: f64,
        hrtf_makeup: f64,
    ) -> Self {
        let view_angle = listener_view.view_angle();
        let (left, right) = stereo_pan_gains(
            listener.direction_to(source),
            view_angle,
            ILD_COEFFICIENT_1_0,
        );
        let level_f32 = gain_as_f32(level);
        Self {
            direction: ListenerRelativeDirection::from_arma(listener, view_angle, source),
            hrtf_gain: gain_as_f32(level * hrtf_makeup),
            fallback_gains: (left * level_f32, right * level_f32),
        }
    }

    /// Renders `buffer` in place: through `hrtf` into `rendered` and back when the sound has an
    /// HRTF source, else by the simple-ILD gains.
    pub(crate) fn render(
        self,
        hrtf: Option<&mut HrtfSource>,
        buffer: &mut StereoBuffer,
        rendered: &mut StereoBuffer,
    ) {
        if let Some(source) = hrtf {
            source.process(buffer, rendered, self.direction, self.hrtf_gain);
            buffer.copy_from(rendered);
        } else {
            let (left, right) = self.fallback_gains;
            apply_stereo_gain(buffer, left, right);
        }
    }
}
