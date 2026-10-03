//! The service thread: the plugin's 100 ms tick.
//!
//! **Role:** the original's `ServiceThread` (`reference/tfar_0_9_12/ts/src/plugin.cpp:1977-2019`):
//! every tick the game timeout (4 s without game activity ends the game, or lifts the plugin's
//! mutes); every 4 s the sweep (expired players removed, the nickname list and voice facts read
//! again, and in game the players without current data muted in serious mode) and the status
//! metadata; every tick the finished radio sounds freed off the audio thread.
//!
//! **Position:** started by `ts3plugin_init` as the thread `tfar-service`, stopped and joined by
//! `ts3plugin_shutdown`; works through `PluginCore`.
//!
//! **Signals and state:** a stop channel it waits on between ticks, and the times of the last
//! sweep and metadata update; every TeamSpeak call is made with no lock held.
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

/// The time between two ticks.
pub const SERVICE_TICK: Duration = Duration::from_millis(100);

/// The time between two sweeps and between two status metadata updates (`MILLIS_TO_EXPIRE`).
pub const SWEEP_INTERVAL: Duration = Duration::from_secs(4);

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
    let started = Instant::now();
    let mut last_sweep = started;
    let mut last_metadata = started;
    loop {
        match stopped.recv_timeout(SERVICE_TICK) {
            Err(RecvTimeoutError::Timeout) => {}
            Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
        }
        let now = Instant::now();
        let server = core.team_speak().current_server();
        core.check_game_timeout(server, now);
        if now.saturating_duration_since(last_sweep) >= SWEEP_INTERVAL {
            last_sweep = now;
            core.sweep(server, now);
        }
        if now.saturating_duration_since(last_metadata) >= SWEEP_INTERVAL {
            last_metadata = now;
            core.publish_status_metadata(server);
        }
        for connection in core.connections() {
            connection.mixer.release_finished();
        }
    }
}

impl PluginCore {
    /// Ends the game, or lifts the mutes, when the game has been silent for 4 s at `now`.
    pub fn check_game_timeout(&self, server: ServerConnectionHandlerId, now: Instant) {
        let timeout = self.session().check_timeout(now);
        if let Some(timeout) = timeout
            && !self.is_dormant()
        {
            self.time_out_game(server, timeout.ended_a_game);
        }
    }

    /// The 4 s sweep of `server` at `now`: expired players removed, the channel read again, and
    /// in game the players without current data muted while the local player lives in serious
    /// mode.
    pub fn sweep(&self, server: ServerConnectionHandlerId, now: Instant) {
        if !self.team_speak().is_connected(server) {
            return;
        }
        let snapshot = read_channel_snapshot(self.team_speak(), server);
        let is_muting = self.is_in_game() && !self.is_dormant();
        let decision = self.with_state(server, |state| {
            let removed = state.radio.remove_expired(now);
            if !removed.is_empty() {
                tracing::debug!(?removed, "players without news for 20 s were removed");
            }
            state.apply_channel_snapshot(&snapshot);
            match state.my_client() {
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
            }
        });
        if let Some(decision) = decision {
            self.send_mute_decision(server, &decision);
        }
    }
}
