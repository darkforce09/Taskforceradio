//! `PushToTalk`: the local push-to-talk's TeamSpeak side: voice activation, the microphone
//! switch, the broadcast, and the release thread that sends a release after TeamSpeak's
//! push-to-talk delay.
//!
//! **Role:** on a press, save and switch off voice activation, switch the microphone on and
//! broadcast the press (`disableVoiceAndSendCommand`,
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:1689`); on a release, wait the delay read once from
//! `settings.db`, then restore voice activation, switch the microphone off unless voice
//! activation was on, and broadcast the release.
//!
//! **Position:** owned by `PluginCore`; the game link thread calls `press` and `release`, the
//! thread `tfar-push-to-talk-release` runs `run_release_thread` from init to shutdown.
//!
//! **Signals and state:** a `PushToTalkState` behind a `parking_lot::Mutex` with a condvar
//! that wakes the release thread. This lock is held across the TeamSpeak calls of a press or a
//! release, as the original's tangent critical section is, so a press and a release never
//! interleave; no audio callback and no TeamSpeak event takes it, and no server lock is taken
//! while it is held.
//!
//! **Invariants:** a release goes out at most once, never after a later press cancelled it; a
//! game end cancels a release still waiting, restoring voice activation and the microphone at
//! once without the broadcast, so nothing is broadcast once the plugin is dormant; a release
//! still waiting at shutdown is sent before the thread ends.

use std::time::{Duration, Instant};

use parking_lot::{Condvar, Mutex};
use tfar_ts3_host::ServerConnectionHandlerId;

use super::push_to_talk_state::{PendingRelease, PressAction, PushToTalkState};
use crate::team_speak::TeamSpeak;

/// The local push-to-talk; see the module header.
#[derive(Debug)]
pub struct PushToTalk {
    state: Mutex<PushToTalkState>,
    wake: Condvar,
    release_delay: Duration,
}

impl PushToTalk {
    /// A push-to-talk whose release waits `release_delay` (TeamSpeak's `delay_ptt_msecs` when
    /// `delay_ptt` is on, else zero).
    pub fn new(release_delay: Duration) -> Self {
        Self {
            state: Mutex::new(PushToTalkState::new()),
            wake: Condvar::new(),
            release_delay,
        }
    }

    /// The delay a release waits.
    pub fn release_delay(&self) -> Duration {
        self.release_delay
    }

    /// The local player pressed: unless a waiting release is cancelled by it, voice activation
    /// is saved and switched off, the microphone switched on and `command` broadcast.
    pub fn press(
        &self,
        team_speak: &dyn TeamSpeak,
        server: ServerConnectionHandlerId,
        command: &str,
    ) -> PressAction {
        let mut state = self.state.lock();
        let action = state.press();
        if action == PressAction::Activate {
            let is_voice_activation_on = team_speak.is_voice_activation_on(server);
            state.save_voice_activation(is_voice_activation_on);
            if is_voice_activation_on
                && let Err(error) = team_speak.set_voice_activation(server, false)
            {
                tracing::warn!(%error, "voice activation could not be switched off");
            }
            switch_input_and_broadcast(team_speak, server, true, command);
        }
        action
    }

    /// The local player released at `now`: `command` is sent once the delay has passed.
    pub fn release(&self, server: ServerConnectionHandlerId, command: String, now: Instant) {
        let deadline = now.checked_add(self.release_delay).unwrap_or(now);
        self.state.lock().schedule_release(PendingRelease {
            server,
            command,
            deadline,
        });
        self.wake.notify_all();
    }

    /// The release thread's loop: sends each release when it is due, until [`PushToTalk::stop`];
    /// a release still waiting then is sent at once.
    pub fn run_release_thread(&self, team_speak: &dyn TeamSpeak) {
        let mut state = self.state.lock();
        loop {
            if state.is_stopping() {
                if let Some(release) = state.take_any_release() {
                    send_release(team_speak, &release, state.was_voice_activation_on());
                }
                return;
            }
            if let Some(release) = state.take_due_release(Instant::now()) {
                send_release(team_speak, &release, state.was_voice_activation_on());
                continue;
            }
            match state.next_deadline() {
                Some(deadline) => {
                    let _ = self.wake.wait_until(&mut state, deadline);
                }
                None => self.wake.wait(&mut state),
            }
        }
    }

    /// The game ended: a release still waiting is cancelled, so its broadcast never goes out
    /// after the plugin is dormant again; voice activation and the microphone return at once to
    /// what that release would have restored. Without a waiting release nothing happens.
    pub fn cancel_pending_release(&self, team_speak: &dyn TeamSpeak) {
        let mut state = self.state.lock();
        if let Some(release) = state.take_any_release() {
            restore_voice_activation_and_input(
                team_speak,
                release.server,
                state.was_voice_activation_on(),
            );
        }
        drop(state);
        self.wake.notify_all();
    }

    /// Asks the release thread to send what waits and end; the caller joins it.
    pub fn stop(&self) {
        self.state.lock().request_stop();
        self.wake.notify_all();
    }
}

/// The release half of `disableVoiceAndSendCommand`: voice activation back on when it was on,
/// the microphone off unless voice activation keeps it open, and the release broadcast.
fn send_release(
    team_speak: &dyn TeamSpeak,
    release: &PendingRelease,
    was_voice_activation_on: bool,
) {
    restore_voice_activation_and_input(team_speak, release.server, was_voice_activation_on);
    broadcast(team_speak, release.server, &release.command);
}

/// Voice activation back on when it was on at the press, and the microphone off unless voice
/// activation keeps it open.
fn restore_voice_activation_and_input(
    team_speak: &dyn TeamSpeak,
    server: ServerConnectionHandlerId,
    was_voice_activation_on: bool,
) {
    if was_voice_activation_on && let Err(error) = team_speak.set_voice_activation(server, true) {
        tracing::warn!(%error, "voice activation could not be switched back on");
    }
    switch_input(team_speak, server, was_voice_activation_on);
}

/// Switches the microphone to `is_active` and broadcasts `command` to the channel.
fn switch_input_and_broadcast(
    team_speak: &dyn TeamSpeak,
    server: ServerConnectionHandlerId,
    is_active: bool,
    command: &str,
) {
    switch_input(team_speak, server, is_active);
    broadcast(team_speak, server, command);
}

/// Switches the microphone to `is_active`.
fn switch_input(team_speak: &dyn TeamSpeak, server: ServerConnectionHandlerId, is_active: bool) {
    if let Err(error) = team_speak.set_input_active(server, is_active) {
        tracing::warn!(%error, is_active, "the microphone could not be switched");
    }
}

/// Broadcasts the push-to-talk `command` to the channel.
fn broadcast(team_speak: &dyn TeamSpeak, server: ServerConnectionHandlerId, command: &str) {
    if let Err(error) = team_speak.send_plugin_command(server, command) {
        tracing::warn!(%error, "the push-to-talk could not be broadcast");
    }
}
