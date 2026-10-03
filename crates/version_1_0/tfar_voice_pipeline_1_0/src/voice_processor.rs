//! `VoiceProcessor`: one sender's voice frame through the whole 1.0 chain.
//!
//! **Role:** the 1.0 `processVoiceData` (`reference/tfar_1_0/ts/src/plugin.cpp:455-743`) for a
//! frame already copied to stereo: the gates, the voice multiplier, direct speech (muffling,
//! HRTF placement at the distance-curve and cone level, its compressor), the radio compressor
//! and every reception, the global volume; and the captured-voice variant that feeds the
//! `my_radio_voice` stream.
//!
//! **Position:** a module of `tfar_voice_pipeline_1_0`; owned by the plugin app, one per server
//! connection beside its `SessionState`, and called from the post-process and captured-voice
//! callbacks with the session already locked.
//!
//! **Signals and state:** the [`VoiceScratch`] buffers; every DSP object belongs to the sender's
//! `ClientEffects` in the session. Runs on the TeamSpeak audio threads under the server lock.
//!
//! **Invariants:** no allocation on the audio path once the scratch fits the frame, the
//! reception list included; direct speech and radio each run their own compressor once per
//! frame; nothing pans twice; the frame keeps its length.

use core::mem;

use tfar_client_state_1_0::prelude::{SessionState, TeamspeakClientId};
use tfar_playback_mixer::PlaybackMixer;
use tfar_sample_buffer::{StereoBuffer, apply_gain, mix_into_additive};

use crate::direct_speech::render_direct_speech;
use crate::error::Result;
use crate::frame_plan::{FramePlan, PlanInputs};
use crate::radio_paths::render_route;
use crate::sender_gate::{GateVerdict, judge_sender};
use crate::voice_context::{VoiceFrameContext, VoiceOutcome};
use crate::voice_scratch::{VoiceScratch, VoiceScratchCapacities};

/// The playback-mixer stream the local player's voice from speaker radios plays on
/// (`appendPlayback("my_radio_voice", …)`, `plugin.cpp:818`).
pub const OWN_RADIO_VOICE_STREAM: &str = "my_radio_voice";

/// The 1.0 voice chain with its reusable buffers; one per server connection.
#[derive(Debug)]
pub struct VoiceProcessor {
    scratch: VoiceScratch,
}

impl VoiceProcessor {
    /// A processor for TeamSpeak frames of up to `max_frames` frames. Allocates every scratch
    /// buffer: call it off the audio thread.
    pub fn new(max_frames: usize) -> Self {
        Self {
            scratch: VoiceScratch::new(max_frames),
        }
    }

    /// The capacities of the scratch buffers, which frames up to `max_frames` never change.
    pub fn scratch_capacities(&self) -> VoiceScratchCapacities {
        self.scratch.capacities()
    }

    /// Processes the stereo frame `samples` of `sender` in place, as `processVoiceData` does
    /// for the post-process callback. The caller holds the server lock over `session` and
    /// copies TeamSpeak's channels in and out with `tfar_sample_buffer::copy_to_stereo` and
    /// `write_back_from_stereo`.
    pub fn process_sender(
        &mut self,
        session: &mut SessionState,
        context: &VoiceFrameContext<'_>,
        sender: TeamspeakClientId,
        samples: &mut StereoBuffer,
    ) -> VoiceOutcome {
        self.process(session, context, sender, samples, false)
    }

    /// Processes the local player's captured voice for the speaker radios that play it
    /// (`onEditCapturedVoiceDataEvent`, `plugin.cpp:789-819`): in a game, alive, with speakers
    /// and the local player's data, the voice goes through the chain as the local player's,
    /// microphone path (no direct speech, no intercom), and the block to append to the
    /// [`OWN_RADIO_VOICE_STREAM`] is returned. `None` when nothing is to be appended. The
    /// outgoing voice stays as it is: `captured` is only read. `context.sender_metadata` is the
    /// local player's own metadata.
    pub fn process_own_voice_for_speakers(
        &mut self,
        session: &mut SessionState,
        context: &VoiceFrameContext<'_>,
        captured: &StereoBuffer,
    ) -> Option<&StereoBuffer> {
        if !context.is_in_game || !session.game.is_alive || session.game.speakers.is_empty() {
            return None;
        }
        let own = session.clients.own()?.client_id();
        let mut voice = mem::take(&mut self.scratch.own_voice);
        voice.copy_from(captured);
        self.process(session, context, own, &mut voice, true);
        self.scratch.own_voice = voice;
        Some(&self.scratch.own_voice)
    }

    /// [`VoiceProcessor::process_own_voice_for_speakers`], appended to `mixer`'s
    /// [`OWN_RADIO_VOICE_STREAM`]; whether a block was appended.
    ///
    /// # Errors
    ///
    /// [`crate::Error::Mixer`] when the mixer refuses the stream.
    pub fn append_own_voice_for_speakers(
        &mut self,
        session: &mut SessionState,
        context: &VoiceFrameContext<'_>,
        captured: &StereoBuffer,
        mixer: &PlaybackMixer,
    ) -> Result<bool> {
        let Some(voice) = self.process_own_voice_for_speakers(session, context, captured) else {
            return Ok(false);
        };
        mixer.append_stream(OWN_RADIO_VOICE_STREAM, voice.as_slice())?;
        Ok(true)
    }

    /// The chain for one frame of `sender`.
    fn process(
        &mut self,
        session: &mut SessionState,
        context: &VoiceFrameContext<'_>,
        sender: TeamspeakClientId,
        samples: &mut StereoBuffer,
        is_from_microphone: bool,
    ) -> VoiceOutcome {
        if !context.is_in_game {
            return VoiceOutcome::PassThrough;
        }
        let view = match judge_sender(session, context, sender) {
            GateVerdict::PassThrough => return VoiceOutcome::PassThrough,
            GateVerdict::Silence => {
                samples.clear();
                return VoiceOutcome::Silenced;
            }
            GateVerdict::Process(view) => view,
        };
        let inputs = PlanInputs {
            settings: context.settings,
            antennas: context.antennas,
            sender,
            view,
            is_from_microphone,
            now: context.now,
        };
        let scratch = &mut self.scratch;
        let Some(plan) = FramePlan::build(
            session,
            inputs,
            &mut scratch.routes,
            &mut scratch.route_plans,
        ) else {
            return VoiceOutcome::PassThrough;
        };
        let Some(sender_data) = session.clients.by_id_mut(sender) else {
            scratch.release_routes();
            return VoiceOutcome::PassThrough;
        };
        let effects = &mut sender_data.effects;
        let VoiceScratch {
            original,
            radio,
            rendered,
            previous_frame,
            mono_mix,
            routes,
            route_plans,
            ..
        } = &mut *scratch;

        apply_gain(samples.as_mut_slice(), plan.voice_volume_multiplier);
        original.copy_from(samples);
        match &plan.direct_speech {
            Some(direct_speech) => {
                render_direct_speech(effects, direct_speech, samples, previous_frame, rendered);
            }
            None if !plan.is_pure_spectator => samples.clear(),
            None => {}
        }

        if let Some(compressor) = effects.radio_compressor_mut() {
            compressor.process_stereo_i16(original);
        }
        for (route, route_plan) in routes.iter().zip(route_plans.iter()) {
            radio.copy_from(original);
            if render_route(effects, route, route_plan, radio, rendered, mono_mix) {
                mix_into_additive(samples.as_mut_slice(), radio.as_slice());
            }
        }
        apply_gain(samples.as_mut_slice(), plan.global_volume);
        scratch.release_routes();
        VoiceOutcome::Processed
    }
}
