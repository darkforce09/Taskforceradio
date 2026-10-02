//! The direct voice of a sender in voice range: its level branch, then its direction.

use tfar_client_state_0_9::DirectVoiceDsp;
use tfar_sample_buffer::{StereoBuffer, apply_gain};

use super::sender_plan::{DirectVoiceLevel, DirectVoicePlan, DistanceLevel};
use super::sound_placement::place_sound;
use crate::voice_gains::as_stage_gain;

#[cfg(test)]
#[path = "tests/direct_voice.rs"]
mod tests;

/// Renders the direct voice in `voice` by `plan` with the sender's direct-voice sound state:
/// the plain distance gain, the vehicle low-pass or the can't-speak low-pass
/// (`reference/tfar_0_9_12/ts/src/plugin.cpp:2698-2725`), then the direction.
///
/// Returns `false`, with `voice` silenced, when the branch's filter is not made yet: a missing
/// can't-speak low-pass would leave a ×14 gain on unfiltered speech.
pub(crate) fn render_direct_voice(
    plan: &DirectVoicePlan,
    dsp: &mut DirectVoiceDsp,
    voice: &mut StereoBuffer,
    direction_output: &mut StereoBuffer,
) -> bool {
    let has_level = match plan.level {
        DirectVoiceLevel::Heard(DistanceLevel::Plain { gain }) => {
            apply_gain(voice.as_mut_slice(), as_stage_gain(gain));
            true
        }
        DirectVoiceLevel::Heard(DistanceLevel::VehicleMuffled { gain }) => {
            dsp.vehicle_filter_mut().is_some_and(|vehicle| {
                vehicle
                    .filter_mut()
                    .process_stereo_i16(voice, as_stage_gain(gain));
                true
            })
        }
        DirectVoiceLevel::CantSpeak { gain } => {
            dsp.cant_speak_filter_mut().is_some_and(|low_pass| {
                low_pass.process_stereo_i16(voice, as_stage_gain(gain));
                true
            })
        }
    };
    if !has_level {
        voice.clear();
        return false;
    }
    place_sound(
        dsp.hrtf_source_mut(),
        voice,
        direction_output,
        plan.placement,
    );
    true
}
