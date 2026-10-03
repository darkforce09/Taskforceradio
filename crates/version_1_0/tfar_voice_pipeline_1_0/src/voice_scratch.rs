//! `VoiceScratch`: the buffers one voice processor reuses for every frame.
//!
//! **Role:** own every block the 1.0 chain needs besides the frame itself: the compressed
//! original for the radios, the per-reception copy, the HRTF output, the object look-back copy,
//! the captured voice, the radio effects' mono block, the receptions and the route plans, all
//! sized once.
//!
//! **Position:** a module of `tfar_voice_pipeline_1_0`; part of [`crate::VoiceProcessor`], one
//! per server connection.
//!
//! **Signals and state:** the buffers, used on the TeamSpeak audio threads under the server lock
//! that guards the processor. The receptions and route plans are emptied at the end of each
//! frame, still under the lock, so the audio thread never drops the last handle on a radio id
//! or vehicle the session shares and never frees one.
//!
//! **Invariants:** created once off the audio thread for the largest block; a frame no longer
//! than that never grows a buffer.

use tfar_client_state_1_0::prelude::HeardVia;
use tfar_radio_effects_1_0::MonoMixBuffer;
use tfar_sample_buffer::StereoBuffer;

use crate::radio_paths::RoutePlan;

/// The receptions one sender is planned for without growing the reception and plan lists; more
/// grow them once.
const ROUTE_PLAN_CAPACITY: usize = 32;

/// The reusable buffers of one voice processor.
#[derive(Debug)]
pub(crate) struct VoiceScratch {
    /// The frame after the voice multiplier, compressed for the radios (`originalBuffer`).
    pub(crate) original: StereoBuffer,
    /// One reception's copy of `original` (`radio_buffer`).
    pub(crate) radio: StereoBuffer,
    /// The HRTF output.
    pub(crate) rendered: StereoBuffer,
    /// The previous frame, copied for the object look-back.
    pub(crate) previous_frame: StereoBuffer,
    /// The local player's captured voice, processed for the speakers.
    pub(crate) own_voice: StereoBuffer,
    /// The radio effects' mono block.
    pub(crate) mono_mix: MonoMixBuffer,
    /// Every way the local player hears the current sender over radio; empty between frames.
    pub(crate) routes: Vec<HeardVia>,
    /// The plan of each reception of the current frame; empty between frames.
    pub(crate) route_plans: Vec<RoutePlan>,
}

/// The capacities of a processor's buffers, to check that frames never grow them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoiceScratchCapacities {
    /// The frames each stereo scratch block holds without reallocating, in this order:
    /// the original, reception, HRTF, look-back and captured-voice blocks.
    pub stereo_frames: [usize; 5],
    /// The frames the radio effects' mono block holds.
    pub mono_frames: usize,
    /// The receptions the route plan list holds.
    pub route_plans: usize,
    /// The receptions the reception list holds.
    pub routes: usize,
}

impl VoiceScratch {
    /// Buffers for frames of up to `max_frames`. Allocates: call it off the audio thread.
    pub(crate) fn new(max_frames: usize) -> Self {
        let stereo = || {
            let mut buffer = StereoBuffer::default();
            buffer.reserve_frames(max_frames);
            buffer
        };
        Self {
            original: stereo(),
            radio: stereo(),
            rendered: stereo(),
            previous_frame: stereo(),
            own_voice: stereo(),
            mono_mix: MonoMixBuffer::with_frames(max_frames),
            routes: Vec::with_capacity(ROUTE_PLAN_CAPACITY),
            route_plans: Vec::with_capacity(ROUTE_PLAN_CAPACITY),
        }
    }

    /// Empties the reception and route plan lists, keeping their capacity.
    pub(crate) fn release_routes(&mut self) {
        self.routes.clear();
        self.route_plans.clear();
    }

    /// The capacities of every buffer now.
    pub(crate) fn capacities(&self) -> VoiceScratchCapacities {
        VoiceScratchCapacities {
            stereo_frames: [
                self.original.capacity_frames(),
                self.radio.capacity_frames(),
                self.rendered.capacity_frames(),
                self.previous_frame.capacity_frames(),
                self.own_voice.capacity_frames(),
            ],
            mono_frames: self.mono_mix.capacity_frames(),
            route_plans: self.route_plans.capacity(),
            routes: self.routes.capacity(),
        }
    }
}
