//! The audio-path half of direct speech: muffle, place, compress.
//!
//! **Role:** run a [`DirectSpeechPlan`] over one frame with the sender's DSP objects, as
//! `plugin.cpp:552-588` does: the object low-pass with its one-frame look-back, the vehicle or
//! can't-speak low-pass, the placement (HRTF or simple ILD) and the direct-speech compressor.
//!
//! **Position:** a module of the crate's `direct_speech`; called by the voice processor on the
//! TeamSpeak audio thread with the sender's `ClientEffects` borrowed under the session lock.
//!
//! **Signals and state:** the sender's effects and the object look-back buffer they hold; the
//! scratch buffers belong to the processor.
//!
//! **Invariants:** allocates nothing; a DSP object that was never ensured skips its step (a
//! missing low-pass leaves its gain, a missing HRTF source falls back to the simple ILD, a
//! missing compressor leaves the frame uncompressed); the direct-speech compressor runs once.

use tfar_client_state_1_0::prelude::{
    CANT_SPEAK_FILTER_KEY, ClientEffects, ObjectInterceptionFilter, VEHICLE_FILTER_KEY,
    VOICE_HRTF_KEY,
};
use tfar_sample_buffer::StereoBuffer;

use crate::direct_speech::plan::{DirectSpeechPlan, Muffling};
use crate::filter_step::filter_or_gain;

/// The gain of the object low-pass (`processFilterStereo(…, 1.0f, …)`).
const OBJECT_FILTER_GAIN: f32 = 1.0;

/// Runs `plan` over `samples` in place with the sender's `effects`; `previous_frame` receives the
/// object look-back and `rendered` the HRTF output.
pub(crate) fn render_direct_speech(
    effects: &mut ClientEffects,
    plan: &DirectSpeechPlan,
    samples: &mut StereoBuffer,
    previous_frame: &mut StereoBuffer,
    rendered: &mut StereoBuffer,
) {
    match plan.muffling {
        Muffling::Clear => effects.object_interception_buffer_mut().copy_from(samples),
        Muffling::ObjectInterception { object_count } => {
            muffle_by_objects(effects, object_count, samples, previous_frame);
        }
        Muffling::Vehicle { loss, gain } => {
            filter_or_gain(
                effects.vehicle_filter_mut(VEHICLE_FILTER_KEY, loss),
                samples,
                gain,
            );
        }
        Muffling::CantSpeak { gain } => {
            filter_or_gain(
                effects.cant_speak_filter_mut(CANT_SPEAK_FILTER_KEY),
                samples,
                gain,
            );
        }
    }
    plan.placement
        .render(effects.hrtf_source_mut(VOICE_HRTF_KEY), samples, rendered);
    if let Some(compressor) = effects.direct_speech_compressor_mut() {
        compressor.process_stereo_i16(samples);
    }
}

/// The object low-pass with its look-back (`plugin.cpp:557-564`): the filter first runs over
/// the previous frame, its output discarded, so a filter just switched to (another object
/// count) starts warm; the current frame is kept as the next look-back; then the current frame
/// runs through the filter. A sender's first frame has no previous frame and skips the
/// look-back, where the original reads its empty buffer.
fn muffle_by_objects(
    effects: &mut ClientEffects,
    object_count: i32,
    samples: &mut StereoBuffer,
    previous_frame: &mut StereoBuffer,
) {
    previous_frame.copy_from(effects.object_interception_buffer_mut());
    effects.object_interception_buffer_mut().copy_from(samples);
    match effects.object_interception_filter_mut(object_count) {
        Some(ObjectInterceptionFilter::Filter(filter)) => {
            if previous_frame.frames() != 0 {
                filter.process_stereo_i16(previous_frame, OBJECT_FILTER_GAIN);
            }
            filter.process_stereo_i16(samples, OBJECT_FILTER_GAIN);
        }
        Some(ObjectInterceptionFilter::Silence) => samples.clear(),
        None => {}
    }
}
