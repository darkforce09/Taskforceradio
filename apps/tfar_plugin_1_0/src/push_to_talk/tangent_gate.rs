//! [`TangentGate`]: whether a tangent press or a delayed release acts, as the original's
//! `waitingForTangentOff` and `skipTangentOff` flags decide.

#[cfg(test)]
#[path = "tests/tangent_gate.rs"]
mod tests;

/// The flags of the push-to-talk critical section (`CommandProcessor.cpp:356-376`, `:631-644`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TangentGate {
    /// Voice activation was on when the transmission started; it is switched back on after.
    pub was_voice_activation_enabled: bool,
    /// A release is waiting for its delay.
    pub is_waiting_for_release: bool,
    /// The waiting release must do nothing: a press came in while it waited.
    pub is_release_skipped: bool,
}

impl TangentGate {
    /// A press: whether it switches the microphone on and broadcasts. With a release waiting,
    /// it does not; the waiting release is skipped instead, so the microphone stays on.
    pub const fn press(&mut self) -> bool {
        if self.is_waiting_for_release {
            self.is_release_skipped = true;
            false
        } else {
            true
        }
    }

    /// A release starts waiting for its delay.
    pub const fn begin_release(&mut self) {
        self.is_waiting_for_release = true;
    }

    /// The delay of a release ran out: whether it switches the microphone back and broadcasts.
    /// A skipped release only clears the skip and keeps waiting for the next release.
    pub const fn finish_release(&mut self) -> bool {
        if self.is_release_skipped {
            self.is_release_skipped = false;
            false
        } else {
            self.is_waiting_for_release = false;
            true
        }
    }
}
