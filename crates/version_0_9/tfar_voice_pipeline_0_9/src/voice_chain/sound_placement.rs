//! Places a sound in the listener's head: through its HRTF source, or by the simple ILD when it
//! has none.

use tfar_hrtf::HrtfSource;
use tfar_sample_buffer::StereoBuffer;
use tfar_spatial_audio::apply_pan;

use super::sender_plan::SoundPlacement;
use crate::voice_gains::{as_stage_gain, hrtf_makeup_gain};

/// Places `sound` at `placement`, in place.
///
/// With an HRTF source the sound is rendered binaurally into `direction_output` at the HRTF
/// makeup gain and copied back; one panner places it, so no ILD follows. Without one (no HRIR
/// sphere was loaded, or the source is not made yet) the 0.9 simple ILD gains pan it, without
/// makeup.
pub(crate) fn place_sound(
    hrtf_source: Option<&mut HrtfSource>,
    sound: &mut StereoBuffer,
    direction_output: &mut StereoBuffer,
    placement: SoundPlacement,
) {
    match hrtf_source {
        Some(source) => {
            source.process(
                sound,
                direction_output,
                placement.direction,
                as_stage_gain(hrtf_makeup_gain()),
            );
            sound.copy_from(direction_output);
        }
        None => apply_pan(sound, placement.ild_gains),
    }
}
