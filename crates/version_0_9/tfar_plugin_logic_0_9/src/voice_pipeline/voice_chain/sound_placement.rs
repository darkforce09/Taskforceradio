//! Places a sound in the listener's head: through its HRTF source, or by the simple ILD when it
//! has none.

use tfar_audio::hrtf::HrtfSource;
use tfar_audio::sample_buffer::StereoBuffer;
use tfar_audio::spatial_audio::apply_pan;

use super::sender_plan::SoundPlacement;
use crate::voice_pipeline::voice_gains::{as_stage_gain, hrtf_makeup_gain};

/// The level an HRTF source renders a sound at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HrtfLevel {
    /// Direct speech: the HRTF makeup gain, so speech straight ahead keeps the level the
    /// original's ear-delay stage left it.
    WithMakeup,
    /// A loudspeaker: unity, as the 1.0 pipeline renders its speakers; the simple ILD the HRTF
    /// replaces puts a loudspeaker straight ahead at 0.625 already, close to the sphere's front
    /// level.
    Unity,
}

impl HrtfLevel {
    /// The gain the HRTF source renders at.
    fn gain(self) -> f32 {
        match self {
            Self::WithMakeup => as_stage_gain(hrtf_makeup_gain()),
            Self::Unity => 1.0,
        }
    }
}

/// Places `sound` at `placement`, in place.
///
/// With an HRTF source the sound is rendered binaurally into `direction_output` at `level` and
/// copied back; one panner places it, so no ILD follows. Without one (no HRIR sphere was loaded,
/// or the source is not made yet) the 0.9 simple ILD gains pan it, without makeup.
pub(crate) fn place_sound(
    hrtf_source: Option<&mut HrtfSource>,
    sound: &mut StereoBuffer,
    direction_output: &mut StereoBuffer,
    placement: SoundPlacement,
    level: HrtfLevel,
) {
    match hrtf_source {
        Some(source) => {
            source.process(sound, direction_output, placement.direction, level.gain());
            sound.copy_from(direction_output);
        }
        None => apply_pan(sound, placement.ild_gains),
    }
}
