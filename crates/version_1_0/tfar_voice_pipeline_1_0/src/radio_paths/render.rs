//! The audio-path half of the radio paths: one reception's copy through its effect.
//!
//! **Role:** run a [`RoutePlan`] over the compressed copy of a frame with the sender's DSP
//! objects, as the loop body of `processVoiceData` does
//! (`reference/tfar_1_0/ts/src/plugin.cpp:600-736`): the radio effect or phone band-pass of a
//! carried radio; the band-pass, distance and vehicle step, underwater low-pass, personal
//! effect and placement of a speaker radio; the long-range effect of the intercom.
//!
//! **Position:** a module of the crate's `radio_paths`; called by the voice processor on the
//! TeamSpeak audio thread for each reception, before the copy is mixed into the frame.
//!
//! **Signals and state:** the sender's effects, borrowed under the session lock; the scratch
//! buffers belong to the processor.
//!
//! **Invariants:** allocates nothing; a reception whose radio effect or band-pass was never
//! ensured is left out of the mix (a radio heard without its effect would sound like direct
//! speech from nowhere), while a missing vehicle or underwater low-pass leaves only its gain and
//! a missing HRTF source falls back to the simple ILD.

use tfar_client_state_1_0::prelude::{ClientEffects, HeardVia, INTERCOM_RADIO_ID, StereoMode};
use tfar_radio_effects_1_0::{MonoMixBuffer, RadioEffect, process_radio_effect};
use tfar_sample_buffer::{StereoBuffer, apply_gain};

use crate::distance_gain::{CANT_SPEAK_GAIN, SPEAKER_GAIN};
use crate::filter_step::filter_or_gain;
use crate::radio_paths::plan::{CarriedRadioSound, RoutePlan, SpeakerMuffling, SpeakerPlan};

/// The error level of the intercom: always clear (`setErrorLeveL(0.f)`).
const INTERCOM_DISTANCE_ERROR: f64 = 0.0;

/// Runs `plan` for `route` over `buffer` (the compressed copy of the frame) in place; whether
/// the result is to be mixed into the frame. `rendered` receives HRTF output and `mono_mix` is
/// the radio effects' block.
pub(crate) fn render_route(
    effects: &mut ClientEffects,
    route: &HeardVia,
    plan: &RoutePlan,
    buffer: &mut StereoBuffer,
    rendered: &mut StereoBuffer,
    mono_mix: &mut MonoMixBuffer,
) -> bool {
    match *plan {
        RoutePlan::Skipped => false,
        RoutePlan::Carried {
            sound,
            distance_error,
            gain,
        } => render_carried_radio(
            effects,
            route,
            sound,
            EffectRun {
                distance_error,
                gain,
                stereo_mode: route.stereo_mode,
            },
            buffer,
            mono_mix,
        ),
        RoutePlan::Speaker(speaker) => {
            render_speaker(effects, route, &speaker, buffer, rendered, mono_mix)
        }
        RoutePlan::Intercom { gain } => run_effect(
            effects.long_range_radio_mut(INTERCOM_RADIO_ID),
            EffectRun {
                distance_error: INTERCOM_DISTANCE_ERROR,
                gain,
                stereo_mode: StereoMode::Both,
            },
            buffer,
            mono_mix,
        ),
    }
}

/// How one radio effect runs: its error, gain and ear.
#[derive(Clone, Copy, Debug)]
struct EffectRun {
    distance_error: f64,
    gain: f32,
    stereo_mode: StereoMode,
}

/// A carried radio: the effect of its sound, keyed by the reception's radio id.
fn render_carried_radio(
    effects: &mut ClientEffects,
    route: &HeardVia,
    sound: CarriedRadioSound,
    run: EffectRun,
    buffer: &mut StereoBuffer,
    mono_mix: &mut MonoMixBuffer,
) -> bool {
    let key = route.radio_id.as_str();
    match sound {
        CarriedRadioSound::Personal => {
            run_effect(effects.personal_radio_mut(key), run, buffer, mono_mix)
        }
        CarriedRadioSound::UnderWater => {
            run_effect(effects.underwater_radio_mut(key), run, buffer, mono_mix)
        }
        CarriedRadioSound::Airborne => {
            run_effect(effects.airborne_radio_mut(key), run, buffer, mono_mix)
        }
        CarriedRadioSound::LongRange => {
            run_effect(effects.long_range_radio_mut(key), run, buffer, mono_mix)
        }
        CarriedRadioSound::Phone => match effects.phone_filter_mut(key) {
            Some(filter) => {
                filter.process_stereo_i16(buffer, run.gain);
                true
            }
            None => false,
        },
        CarriedRadioSound::Silence => false,
    }
}

/// A speaker radio (`plugin.cpp:677-714`): band-pass at ×4, distance gain or vehicle low-pass,
/// the underwater low-pass at ×14, the personal radio effect, then the placement.
fn render_speaker(
    effects: &mut ClientEffects,
    route: &HeardVia,
    speaker: &SpeakerPlan,
    buffer: &mut StereoBuffer,
    rendered: &mut StereoBuffer,
    mono_mix: &mut MonoMixBuffer,
) -> bool {
    let key = route.radio_id.as_str();
    let Some(band_pass) = effects.speaker_filter_mut(key) else {
        return false;
    };
    band_pass.process_stereo_i16(buffer, SPEAKER_GAIN);
    match speaker.muffling {
        SpeakerMuffling::Clear { gain } => apply_gain(buffer.as_mut_slice(), gain),
        SpeakerMuffling::Vehicle { loss, gain } => {
            filter_or_gain(effects.vehicle_filter_mut(key, loss), buffer, gain);
        }
    }
    if speaker.is_under_water {
        filter_or_gain(effects.cant_speak_filter_mut(key), buffer, CANT_SPEAK_GAIN);
    }
    let run = EffectRun {
        distance_error: speaker.distance_error,
        gain: speaker.effect_gain,
        stereo_mode: route.stereo_mode,
    };
    if !run_effect(effects.personal_radio_mut(key), run, buffer, mono_mix) {
        return false;
    }
    speaker
        .placement
        .render(effects.hrtf_source_mut(key), buffer, rendered);
    true
}

/// Sets `effect` to the run's error and runs it over `buffer`; `false` when the effect was
/// never ensured.
fn run_effect<E: RadioEffect>(
    effect: Option<&mut E>,
    run: EffectRun,
    buffer: &mut StereoBuffer,
    mono_mix: &mut MonoMixBuffer,
) -> bool {
    let Some(effect) = effect else {
        return false;
    };
    effect.set_error_level(run.distance_error);
    process_radio_effect(effect, buffer, run.gain, run.stereo_mode, mono_mix);
    true
}
