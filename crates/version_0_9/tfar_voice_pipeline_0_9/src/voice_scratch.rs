//! `VoiceScratch`: every buffer the voice chain works in, made once per server connection.
//!
//! **Role:** own the stereo, mono and route buffers the voice chain reuses from frame to frame,
//! sized up front for the largest frame expected, and report their capacities.
//!
//! **Position:** a field of `VoiceProcessor`; made with it on the plugin's connect handling,
//! used by `voice_chain/` and `own_voice.rs` on the TeamSpeak audio thread.
//!
//! **Signals and state:** the buffers' contents, valid only within one frame. Touched only by
//! the thread that holds the processor, inside the server lock.
//!
//! **Invariants:** a frame of at most the frames the scratch was made for, with at most
//! [`ROUTE_CAPACITY`] receptions and a sender nickname of at most [`NICKNAME_CAPACITY`] bytes,
//! changes no capacity.

use tfar_radio_effects_0_9::MonoMixBuffer;
use tfar_sample_buffer::StereoBuffer;

use crate::voice_chain::PlannedRoute;

#[cfg(test)]
#[path = "tests/voice_scratch.rs"]
mod tests;

/// The frames the scratch is made for by default: 100 ms at 48 kHz, ten of TeamSpeak's usual
/// 10 ms voice frames.
pub const DEFAULT_MAX_FRAMES: usize = 4_800;

/// The receptions of one sender the route list holds without growing.
pub const ROUTE_CAPACITY: usize = 32;

/// The bytes of a sender nickname the scratch holds without growing; TeamSpeak nicknames are at
/// most 30 characters, up to 120 bytes of UTF-8.
pub const NICKNAME_CAPACITY: usize = 128;

/// The buffers one server connection's voice chain reuses.
#[derive(Debug)]
pub struct VoiceScratch {
    /// The sender's voice after the multiplier: the input of every radio reception.
    pub(crate) original: StereoBuffer,
    /// One radio reception being processed.
    pub(crate) radio: StereoBuffer,
    /// The output of an HRTF source, copied back into the sound it placed.
    pub(crate) direction_output: StereoBuffer,
    /// The local player's own voice on its way to the loudspeakers.
    pub(crate) own_voice: StereoBuffer,
    /// The mono block a radio effect runs on.
    pub(crate) mono_mix: MonoMixBuffer,
    /// The receptions of the current sender, planned before the sound state is borrowed.
    pub(crate) routes: Vec<PlannedRoute>,
    /// The current sender's nickname, which keys their entry once the plan is made.
    pub(crate) sender_nickname: String,
}

/// The capacities of every scratch buffer, to compare before and after frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScratchCapacities {
    /// The frames of the original, radio, direction and own-voice buffers, in that order.
    pub stereo_frames: [usize; 4],
    /// The frames of the mono effect block.
    pub mono_frames: usize,
    /// The receptions the route list holds.
    pub routes: usize,
    /// The bytes the nickname holds.
    pub nickname_bytes: usize,
}

impl VoiceScratch {
    /// Scratch for frames of up to `max_frames` frames. Allocates: call it off the audio
    /// thread.
    pub fn with_frames(max_frames: usize) -> Self {
        Self {
            original: StereoBuffer::with_frames(max_frames),
            radio: StereoBuffer::with_frames(max_frames),
            direction_output: StereoBuffer::with_frames(max_frames),
            own_voice: StereoBuffer::with_frames(max_frames),
            mono_mix: MonoMixBuffer::with_frames(max_frames),
            routes: Vec::with_capacity(ROUTE_CAPACITY),
            sender_nickname: String::with_capacity(NICKNAME_CAPACITY),
        }
    }

    /// The capacity of every buffer now.
    pub fn capacities(&self) -> ScratchCapacities {
        ScratchCapacities {
            stereo_frames: [
                self.original.capacity_frames(),
                self.radio.capacity_frames(),
                self.direction_output.capacity_frames(),
                self.own_voice.capacity_frames(),
            ],
            mono_frames: self.mono_mix.capacity_frames(),
            routes: self.routes.capacity(),
            nickname_bytes: self.sender_nickname.capacity(),
        }
    }
}

impl Default for VoiceScratch {
    fn default() -> Self {
        Self::with_frames(DEFAULT_MAX_FRAMES)
    }
}
