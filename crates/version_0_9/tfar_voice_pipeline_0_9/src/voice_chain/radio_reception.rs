//! One reception of the sender's radio: the effect of the sender's subtype, then the
//! loudspeaker stages when the radio plays on a loudspeaker.
//!
//! **Role:** run one planned reception over its copy of the compressed voice: the personal,
//! long-range or underwater effect, the phone band-pass or silence
//! (`reference/tfar_0_9_12/ts/src/plugin.cpp:2735-2759`), then the loudspeaker's band-pass,
//! distance level, under-water low-pass and direction (plugin.cpp:2762-2791).
//!
//! **Position:** called by `frame_render.rs` once per reception, after the compressor; borrows
//! the reception's `RadioDsp` from the sender's sound state.
//!
//! **Signals and state:** advances the effect, filter and HRTF state of one radio id on the
//! TeamSpeak audio thread; makes nothing.
//!
//! **Invariants:** a reception whose sound object is not made yet is left out of the mix and
//! reported, never run unfiltered; a silent subtype mixes nothing; the loudspeaker band-pass
//! runs before the level, the under-water low-pass after it, the direction last.

use tfar_client_state_0_9::{RadioDsp, RadioEffectKind};
use tfar_radio_effects_0_9::{MonoMixBuffer, RadioEffect, process_radio_effect};
use tfar_sample_buffer::{StereoBuffer, apply_gain};

use super::sender_plan::{DistanceLevel, LoudspeakerPlan, PlannedRoute};
use super::sound_placement::place_sound;
use crate::voice_gains::{CANT_SPEAK_GAIN, SPEAKER_GAIN, as_stage_gain};

#[cfg(test)]
#[path = "tests/radio_reception.rs"]
mod tests;

/// What one reception gave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReceptionResult {
    /// The reception's sound is in the radio buffer, to mix.
    Mixed,
    /// The sender's subtype makes no sound; nothing to mix.
    Silent,
    /// A sound object the reception needs is not made yet; nothing to mix.
    MissingSoundState,
}

/// The scratch one reception renders in.
pub(crate) struct ReceptionBuffers<'scratch> {
    /// The reception's copy of the compressed voice, processed in place.
    pub(crate) radio: &'scratch mut StereoBuffer,
    /// The mono block of the radio effect.
    pub(crate) mono_mix: &'scratch mut MonoMixBuffer,
    /// The output block of an HRTF source.
    pub(crate) direction_output: &'scratch mut StereoBuffer,
}

/// Renders `route` with `effect`, the effect of the sender's subtype, using the reception's
/// sound state `dsp`.
pub(crate) fn render_reception(
    route: &PlannedRoute,
    effect: RadioEffectKind,
    dsp: Option<&mut RadioDsp>,
    mut buffers: ReceptionBuffers<'_>,
) -> ReceptionResult {
    if effect == RadioEffectKind::Silent {
        return ReceptionResult::Silent;
    }
    let Some(dsp) = dsp else {
        return ReceptionResult::MissingSoundState;
    };
    let gain = as_stage_gain(route.effect_gain);
    let has_effect = match effect {
        RadioEffectKind::Personal => {
            run_effect(dsp.personal_effect_mut(), route, gain, &mut buffers)
        }
        RadioEffectKind::LongRange => {
            run_effect(dsp.long_range_effect_mut(), route, gain, &mut buffers)
        }
        RadioEffectKind::UnderWater => {
            run_effect(dsp.under_water_effect_mut(), route, gain, &mut buffers)
        }
        RadioEffectKind::Phone => dsp.phone_filter_mut().is_some_and(|band_pass| {
            band_pass.process_stereo_i16(buffers.radio, gain);
            true
        }),
        RadioEffectKind::Silent => false,
    };
    if !has_effect {
        return ReceptionResult::MissingSoundState;
    }
    match &route.loudspeaker {
        Some(loudspeaker) if !render_loudspeaker(loudspeaker, dsp, &mut buffers) => {
            ReceptionResult::MissingSoundState
        }
        _ => ReceptionResult::Mixed,
    }
}

/// Runs `effect` over the radio buffer at `gain` into the route's ear, at the route's error
/// level; `false` when the effect is not made yet.
fn run_effect<E: RadioEffect>(
    effect: Option<&mut E>,
    route: &PlannedRoute,
    gain: f32,
    buffers: &mut ReceptionBuffers<'_>,
) -> bool {
    let Some(effect) = effect else {
        return false;
    };
    effect.set_error_level(route.effect_error);
    process_radio_effect(
        effect,
        buffers.radio,
        gain,
        route.stereo_mode,
        buffers.mono_mix,
    );
    true
}

/// The loudspeaker stages over the radio buffer; `false` when a filter is not made yet.
fn render_loudspeaker(
    plan: &LoudspeakerPlan,
    dsp: &mut RadioDsp,
    buffers: &mut ReceptionBuffers<'_>,
) -> bool {
    let Some(band_pass) = dsp.speaker_filter_mut() else {
        return false;
    };
    band_pass.process_stereo_i16(buffers.radio, as_stage_gain(SPEAKER_GAIN));
    match plan.level {
        DistanceLevel::Plain { gain } => {
            apply_gain(buffers.radio.as_mut_slice(), as_stage_gain(gain));
        }
        DistanceLevel::VehicleMuffled { gain } => {
            let Some(vehicle) = dsp.vehicle_filter_mut() else {
                return false;
            };
            vehicle
                .filter_mut()
                .process_stereo_i16(buffers.radio, as_stage_gain(gain));
        }
    }
    if plan.is_under_water {
        let Some(low_pass) = dsp.under_water_filter_mut() else {
            return false;
        };
        low_pass.process_stereo_i16(buffers.radio, as_stage_gain(CANT_SPEAK_GAIN));
    }
    place_sound(
        dsp.hrtf_source_mut(),
        buffers.radio,
        buffers.direction_output,
        plan.placement,
    );
    true
}
