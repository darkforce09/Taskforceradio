//! The service thread: the plugin's 100 ms tick.
//!
//! **Role:** the original's `ServiceThread` (`reference/tfar_0_9_12/ts/src/plugin.cpp:1977-2019`):
//! every tick the game timeout (4 s without game activity ends the game and puts the plugin back
//! to dormant, or lifts the plugin's mutes); every 4 s the sweep (expired players removed, the nickname list and voice facts read
//! again, and in game the players without current data muted in serious mode) and the status
//! metadata; every second the audio callbacks' lock waits and incomplete frames logged when any
//! occurred; every tick the finished radio sounds freed off the audio thread.
//!
//! **Position:** started by `ts3plugin_init` as the thread `tfar-service`, stopped and joined by
//! `ts3plugin_shutdown`; works through `PluginCore`.
//!
//! **Signals and state:** a stop channel it waits on between ticks, and the times of the last
//! sweep, metadata update and diagnostics log; every TeamSpeak call is made with no lock held.
//!
//! **Invariants:** a dormant plugin makes no mute request and writes no metadata; the thread
//! ends within one tick of the stop signal.

use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use tfar_radio_routing_0_9::{MuteDecision, is_serious_mode, set_mute_for_dead_players};
use tfar_ts3_host::ServerConnectionHandlerId;

use crate::channel_snapshot::read_channel_snapshot;
use crate::plugin_core::PluginCore;

#[cfg(test)]
#[path = "tests/service_thread.rs"]
mod tests;

/// The time between two ticks.
pub const SERVICE_TICK: Duration = Duration::from_millis(100);

/// The time between two sweeps and between two status metadata updates (`MILLIS_TO_EXPIRE`).
pub const SWEEP_INTERVAL: Duration = Duration::from_secs(4);

/// The time between two logs of the audio diagnostics; a quiet second logs nothing.
pub const AUDIO_DIAGNOSTICS_INTERVAL: Duration = Duration::from_secs(1);

/// The thread's name.
const SERVICE_THREAD_NAME: &str = "tfar-service";

/// The running service thread.
#[derive(Debug)]
pub struct ServiceThread {
    stop: Sender<()>,
    thread: JoinHandle<()>,
}

impl ServiceThread {
    /// Starts the tick over `core`.
    ///
    /// # Errors
    ///
    /// The operating system's error when the thread cannot start.
    pub fn start(core: Arc<PluginCore>) -> std::io::Result<Self> {
        let (stop, stopped) = crossbeam_channel::bounded(1);
        let thread = std::thread::Builder::new()
            .name(SERVICE_THREAD_NAME.to_owned())
            .spawn(move || run_service(&core, &stopped))?;
        Ok(Self { stop, thread })
    }

    /// Stops the tick and joins the thread.
    pub fn stop(self) {
        let _ = self.stop.try_send(());
        if self.thread.join().is_err() {
            tracing::warn!("the service thread ended abnormally");
        }
    }
}

/// The tick loop until a stop arrives or the sender goes.
fn run_service(core: &PluginCore, stopped: &Receiver<()>) {
    let mut schedule = ServiceSchedule::new(Instant::now());
    loop {
        match stopped.recv_timeout(SERVICE_TICK) {
            Err(RecvTimeoutError::Timeout) => {}
            Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
        }
        core.service_tick(&mut schedule, Instant::now());
    }
}

/// When the service tick last swept, wrote the status metadata and logged the audio
/// diagnostics, and the voice frames without sound state counted at that log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServiceSchedule {
    last_sweep: Instant,
    last_metadata: Instant,
    last_audio_diagnostics: Instant,
    logged_frames_missing_sound_state: u64,
}

impl ServiceSchedule {
    /// A schedule whose first sweep and metadata update fall [`SWEEP_INTERVAL`] after `now`.
    pub const fn new(now: Instant) -> Self {
        Self {
            last_sweep: now,
            last_metadata: now,
            last_audio_diagnostics: now,
            logged_frames_missing_sound_state: 0,
        }
    }
}

impl PluginCore {
    /// One service tick at `now`: the game timeout, the sweep and the metadata when `schedule`
    /// says they are due, and the finished radio sounds freed. The service thread calls it every
    /// [`SERVICE_TICK`]; tests call it with a made-up `now`.
    pub fn service_tick(&self, schedule: &mut ServiceSchedule, now: Instant) {
        let server = self.team_speak().current_server();
        self.check_game_timeout(server, now);
        if now.saturating_duration_since(schedule.last_sweep) >= SWEEP_INTERVAL {
            schedule.last_sweep = now;
            self.sweep(server, now);
        }
        if now.saturating_duration_since(schedule.last_metadata) >= SWEEP_INTERVAL {
            schedule.last_metadata = now;
            self.publish_status_metadata(server);
        }
        if now.saturating_duration_since(schedule.last_audio_diagnostics)
            >= AUDIO_DIAGNOSTICS_INTERVAL
        {
            schedule.last_audio_diagnostics = now;
            self.log_audio_diagnostics(server, schedule);
        }
        for connection in self.connections() {
            connection.mixer.release_finished();
        }
    }
    /// Ends the game, or lifts the mutes, when the game has been silent for 4 s at `now`.
    pub fn check_game_timeout(&self, server: ServerConnectionHandlerId, now: Instant) {
        let timeout = self.session().check_timeout(now);
        if let Some(timeout) = timeout
            && !self.is_dormant()
        {
            self.time_out_game(server, timeout.ended_a_game);
        }
    }

    /// The 4 s sweep of `server` at `now`: expired players removed (and freed after the lock is
    /// released), the channel read again, and in game the players without current data muted
    /// while the local player lives in serious mode.
    pub fn sweep(&self, server: ServerConnectionHandlerId, now: Instant) {
        if !self.team_speak().is_connected(server) {
            return;
        }
        let snapshot = read_channel_snapshot(self.team_speak(), server);
        let is_muting = self.is_in_game() && !self.is_dormant();
        let swept = self.with_state(server, |state| {
            let removed = state.radio.remove_expired(now);
            state.apply_channel_snapshot(&snapshot);
            let decision = match state.my_client() {
                Some(me) if is_muting => {
                    let is_serious =
                        is_serious_mode(&snapshot.channel_name, &state.radio.serious_channel_name);
                    set_mute_for_dead_players(
                        &state.radio,
                        &mut state.mutes,
                        &snapshot.member_ids(),
                        me,
                        is_serious,
                    )
                }
                _ => MuteDecision::default(),
            };
            (removed, decision)
        });
        let Some((removed, decision)) = swept else {
            return;
        };
        if !removed.is_empty() {
            let nicknames: Vec<&str> = removed
                .iter()
                .map(|(nickname, _)| nickname.as_str())
                .collect();
            tracing::debug!(?nicknames, "players without news for 20 s were removed");
        }
        // Their sound state is freed here, outside the lock the audio thread waits for.
        drop(removed);
        self.send_mute_decision(server, &decision);
    }
}

impl PluginCore {
    /// Logs, when any is non-zero, the audio callbacks' lock waits and skipped blocks and the
    /// voice frames of `server` without sound state since the last log.
    fn log_audio_diagnostics(
        &self,
        server: ServerConnectionHandlerId,
        schedule: &mut ServiceSchedule,
    ) {
        let counts = self.audio_diagnostics().take();
        let frames_missing_sound_state = self
            .with_state(server, |state| state.voice.frames_missing_sound_state())
            .unwrap_or(schedule.logged_frames_missing_sound_state);
        let new_frames_missing_sound_state =
            frames_missing_sound_state.saturating_sub(schedule.logged_frames_missing_sound_state);
        schedule.logged_frames_missing_sound_state = frames_missing_sound_state;
        if counts.is_zero() && new_frames_missing_sound_state == 0 {
            return;
        }
        tracing::debug!(
            contended_state_locks = counts.contended_state_locks,
            skipped_blocks = counts.skipped_blocks,
            frames_missing_sound_state = new_frames_missing_sound_state,
            "audio: callback lock waits and incomplete frames in the last second"
        );
    }
}
