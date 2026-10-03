//! [`TangentGate`]: whether a tangent press or a delayed release acts, as the original's
//! `waitingForTangentOff` and `skipTangentOff` flags decide, and the game end that cancels every
//! release still waiting.

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
    /// Counts the game ends; a release begun before the latest one never acts.
    pub release_generation: u64,
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

    /// A release starts waiting for its delay; returns the generation it belongs to, which
    /// [`finish_release`](Self::finish_release) is given when the delay runs out.
    pub const fn begin_release(&mut self) -> u64 {
        self.is_waiting_for_release = true;
        self.release_generation
    }

    /// The delay of a release of `generation` ran out: whether it switches the microphone back
    /// and broadcasts. A release from before the latest game end does nothing; a skipped release
    /// only clears the skip and keeps waiting for the next release.
    pub const fn finish_release(&mut self, generation: u64) -> bool {
        if generation != self.release_generation {
            false
        } else if self.is_release_skipped {
            self.is_release_skipped = false;
            false
        } else {
            self.is_waiting_for_release = false;
            true
        }
    }

    /// The game ended: every release still waiting is cancelled, so none acts once the plugin
    /// is dormant, and the next press starts afresh.
    pub const fn cancel_releases(&mut self) {
        self.is_waiting_for_release = false;
        self.is_release_skipped = false;
        self.release_generation = self.release_generation.wrapping_add(1);
    }
}
