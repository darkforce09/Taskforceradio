//! `PushToTalkState`: the local push-to-talk between a press, a delayed release and the next
//! press, as a pure state machine.
//!
//! **Role:** decide what a press does while a delayed release is still waiting, and when a
//! release is due. The original keeps this in two flags and one thread per release
//! (`skip_tangent_off`, `waiting_tangent_off`, `process_tangent_off`,
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:1703-1726`).
//!
//! **Position:** held by `PushToTalk` behind its mutex; the game link thread presses and
//! schedules, the release thread takes what is due.
//!
//! **Signals and state:** at most one pending release, the voice activation state saved at the
//! press, and the stop flag; time comes in as `now`.
//!
//! **Invariants:** a press while a release waits cancels that release and sends nothing, so the
//! other players, who never saw the release, keep hearing the press; a cancelled release never
//! fires later, so a release is never sent while the player holds the key again.

use std::time::Instant;

use tfar_ts3_host::ServerConnectionHandlerId;

#[cfg(test)]
#[path = "tests/push_to_talk_state.rs"]
mod tests;

/// A release of the local push-to-talk, waiting for its delay to pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingRelease {
    /// The server connection the push-to-talk belongs to.
    pub server: ServerConnectionHandlerId,
    /// The `TANGENT…\tRELEASED…` plugin command to broadcast.
    pub command: String,
    /// When the release is sent.
    pub deadline: Instant,
}

/// What a press of the local push-to-talk does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PressAction {
    /// No release was waiting: save and switch off voice activation, switch the microphone on
    /// and broadcast the press.
    Activate,
    /// A release was waiting and is now cancelled: nothing is sent, the push-to-talk simply
    /// stays on.
    CancelledPendingRelease,
}

/// The local push-to-talk; see the module header.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PushToTalkState {
    pending_release: Option<PendingRelease>,
    is_voice_activation_saved_on: bool,
    is_stopping: bool,
}

impl PushToTalkState {
    /// A push-to-talk that is up, with nothing waiting.
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies a press.
    pub fn press(&mut self) -> PressAction {
        if self.pending_release.take().is_some() {
            PressAction::CancelledPendingRelease
        } else {
            PressAction::Activate
        }
    }

    /// Records the voice activation state found at an activating press, to restore at release.
    pub fn save_voice_activation(&mut self, is_on: bool) {
        self.is_voice_activation_saved_on = is_on;
    }

    /// Whether voice activation was on at the last activating press.
    pub fn was_voice_activation_on(&self) -> bool {
        self.is_voice_activation_saved_on
    }

    /// Schedules `release`, replacing one still waiting.
    pub fn schedule_release(&mut self, release: PendingRelease) {
        self.pending_release = Some(release);
    }

    /// The waiting release's deadline.
    pub fn next_deadline(&self) -> Option<Instant> {
        self.pending_release
            .as_ref()
            .map(|release| release.deadline)
    }

    /// Takes the waiting release when its deadline has passed at `now`.
    pub fn take_due_release(&mut self, now: Instant) -> Option<PendingRelease> {
        if self.next_deadline().is_some_and(|deadline| deadline <= now) {
            self.pending_release.take()
        } else {
            None
        }
    }

    /// Takes the waiting release whatever its deadline: the plugin is stopping and sends it now.
    pub fn take_any_release(&mut self) -> Option<PendingRelease> {
        self.pending_release.take()
    }

    /// Asks the release thread to stop.
    pub fn request_stop(&mut self) {
        self.is_stopping = true;
    }

    /// Whether the release thread is asked to stop.
    pub fn is_stopping(&self) -> bool {
        self.is_stopping
    }
}
