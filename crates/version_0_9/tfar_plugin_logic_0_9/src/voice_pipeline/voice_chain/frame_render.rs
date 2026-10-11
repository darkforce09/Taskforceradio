//! One sender's frame through the chain: multiplier, direct voice, compressor, every
//! reception, global volume.
//!
//! **Role:** run a planned frame over the sender's sound state in the original's order
//! (`reference/tfar_0_9_12/ts/src/plugin.cpp:2687-2802`): the voice volume multiplier, a copy of
//! the voice for the radio, the direct voice or silence, one compressor pass over the copy, each
//! reception on its own copy mixed in with saturation, then the global volume.
//!
//! **Position:** called by `VoiceProcessor` once the plan is made and the sender's entry is
//! borrowed mutably; calls `direct_voice.rs` and `radio_reception.rs`.
//!
//! **Signals and state:** advances the sender's compressor, filters, effects and HRTF sources
//! and writes the scratch buffers, on the TeamSpeak audio thread; allocates nothing for frames
//! within the scratch's size.
//!
//! **Invariants:** the compressor runs exactly once per frame, whatever the number of
//! receptions; each reception starts from the same compressed copy; the mix saturates; the
//! global volume is the last gain.

use crate::client_state::ClientDsp;
use tfar_audio::sample_buffer::{StereoBuffer, apply_gain, mix_into_additive};

use super::direct_voice::render_direct_voice;
use super::radio_reception::{ReceptionBuffers, ReceptionResult, render_reception};
use super::sender_plan::SenderPlan;
use crate::voice_pipeline::voice_gains::as_stage_gain;
use crate::voice_pipeline::voice_scratch::VoiceScratch;

/// Renders `samples`, the sender's frame, by `plan` with the sender's sound state `dsp` and the
/// receptions planned into `scratch`. Returns `false` when a sound object the frame needed was
/// not made yet; that path stayed silent.
pub(crate) fn render_frame(
    plan: &SenderPlan,
    dsp: &mut ClientDsp,
    samples: &mut StereoBuffer,
    scratch: &mut VoiceScratch,
) -> bool {
    let VoiceScratch {
        original,
        radio,
        direction_output,
        mono_mix,
        routes,
        ..
    } = scratch;
    apply_gain(
        samples.as_mut_slice(),
        as_stage_gain(plan.voice_volume_multiplier),
    );
    original.copy_from(samples);
    let mut is_complete = true;
    match &plan.direct_voice {
        Some(direct_voice) => {
            is_complete &= render_direct_voice(
                direct_voice,
                dsp.direct_voice_mut(),
                samples,
                direction_output,
            );
        }
        None => samples.clear(),
    }
    match dsp.compressor_mut() {
        Some(compressor) => compressor.process_stereo_i16(original),
        None => is_complete = false,
    }
    for route in routes.iter() {
        radio.copy_from(original);
        let buffers = ReceptionBuffers {
            radio,
            mono_mix,
            direction_output,
        };
        match render_reception(route, plan.effect, dsp.radio_mut(&route.radio_id), buffers) {
            ReceptionResult::Mixed => mix_into_additive(samples.as_mut_slice(), radio.as_slice()),
            ReceptionResult::Silent => {}
            ReceptionResult::MissingSoundState => is_complete = false,
        }
    }
    apply_gain(samples.as_mut_slice(), as_stage_gain(plan.global_volume));
    is_complete
}
