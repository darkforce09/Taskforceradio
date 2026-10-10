//! [`ServiceThread`]: the plugin's 100 ms housekeeping loop.
//!
//! **Role:** `ServiceThread` of `reference/tfar_1_0/ts/src/plugin.cpp:133-163`: the game link's
//! timeout and transitions every tick; every 4 s the serious-channel move while in game (gated by
//! Arma's focus unless `moveWhileTabbedOut`, and by `noAutomoveSpectator`), the expiry sweep with
//! the mute refresh, the client metadata, the DSP objects and the own metadata; every 500 ms a
//! debug window snapshot while the window is open; every second the audio callbacks' lock waits
//! logged when any occurred; finished radio sounds are freed every tick.
//! The periods come from `ServicePeriods`, which a test may shorten.
//!
//! **Position:** started in `ts3plugin_init`, stopped and joined in `ts3plugin_shutdown`.
//!
//! **Signals and state:** the thread `tfar-service` with a weak runtime handle and a stop channel;
//! it takes the same locks as the handlers, in the same order.
//!
//! **Invariants:** a stop request ends the loop within one tick; a dormant plugin only polls the
//! game link.

use std::sync::Weak;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::{RecvTimeoutError, Sender};

use crate::diagnostics::debug_snapshot::publish_debug_snapshot;
use crate::runtime::PluginRuntime;
use crate::service::service_periods::ServicePeriods;

/// The name of the service thread.
pub const SERVICE_THREAD_NAME: &str = "tfar-service";

/// The time between two logs of the audio callbacks' lock waits; a quiet second logs nothing.
const AUDIO_DIAGNOSTICS_PERIOD: Duration = Duration::from_secs(1);

/// The running service thread.
#[derive(Debug)]
pub struct ServiceThread {
    stop: Sender<()>,
    thread: JoinHandle<()>,
}

impl ServiceThread {
    /// Starts the loop over `runtime` with `periods`.
    ///
    /// # Errors
    ///
    /// The system's error when the thread cannot start.
    pub fn start(runtime: Weak<PluginRuntime>, periods: ServicePeriods) -> std::io::Result<Self> {
        let (stop, stop_receiver) = crossbeam_channel::bounded::<()>(0);
        let thread = std::thread::Builder::new()
            .name(SERVICE_THREAD_NAME.to_owned())
            .spawn(move || {
                let mut timers = Timers::default();
                loop {
                    match stop_receiver.recv_timeout(periods.tick) {
                        Err(RecvTimeoutError::Timeout) => {}
                        Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
                    }
                    let Some(runtime) = runtime.upgrade() else {
                        return;
                    };
                    tick(&runtime, &periods, &mut timers, Instant::now());
                }
            })?;
        Ok(Self { stop, thread })
    }

    /// Stops the loop and joins it.
    pub fn stop(self) {
        drop(self.stop);
        if self.thread.join().is_err() {
            tracing::error!(thread = SERVICE_THREAD_NAME, "a thread panicked");
        }
    }
}

/// When the periodic work last ran.
#[derive(Debug, Default)]
struct Timers {
    sweep: Option<Instant>,
    debug_snapshot: Option<Instant>,
    audio_diagnostics: Option<Instant>,
}

/// Whether `period` has passed since `last`, updating `last` when it has.
fn is_due(last: &mut Option<Instant>, period: Duration, now: Instant) -> bool {
    let is_due = last.is_none_or(|at| now.saturating_duration_since(at) >= period);
    if is_due {
        *last = Some(now);
    }
    is_due
}

/// One tick of the loop.
fn tick(runtime: &PluginRuntime, periods: &ServicePeriods, timers: &mut Timers, now: Instant) {
    profiling::scope!("service_tick");
    if runtime.is_shutting_down() {
        return;
    }
    runtime.poll_game_link(now);
    runtime.sounds.release_finished();
    if is_due(&mut timers.audio_diagnostics, AUDIO_DIAGNOSTICS_PERIOD, now) {
        log_audio_diagnostics(runtime);
    }
    if runtime.is_dormant() {
        return;
    }
    if is_due(&mut timers.sweep, periods.sweep, now) {
        sweep(runtime, now);
    }
    if is_due(&mut timers.debug_snapshot, periods.debug_snapshot, now) {
        publish_debug_snapshot(runtime);
    }
}

/// Logs the audio callbacks' lock waits and skipped blocks since the last log, when any occurred.
fn log_audio_diagnostics(runtime: &PluginRuntime) {
    let counts = runtime.audio_diagnostics.take();
    if !counts.is_zero() {
        tracing::debug!(
            contended_state_locks = counts.contended_state_locks,
            skipped_blocks = counts.skipped_blocks,
            "audio: callback lock waits in the last second"
        );
    }
}

/// The 4 s work on the server in front.
fn sweep(runtime: &PluginRuntime, now: Instant) {
    let ts = runtime.ts();
    let server = ts.current_server();
    if !ts.is_connected(server) {
        return;
    }
    let settings = runtime.settings();
    if runtime.is_in_game() {
        let is_spectating = runtime.servers.get(server).is_some_and(|state| {
            state
                .lock()
                .session
                .clients
                .own()
                .is_some_and(|own| own.is_spectating)
        });
        if !(settings.no_automove_spectator() && is_spectating) {
            runtime.move_to_serious_channel(server, true);
        }
    }
    if let Some(state) = runtime.servers.get(server) {
        let expired = state.lock().session.sweep_expired(now);
        if !expired.is_empty() {
            tracing::debug!(count = expired.len(), "clients expired");
        }
    }
    runtime.refresh_every_mute(server);
    runtime.refresh_client_metadata(server);
    if let Some(state) = runtime.servers.get(server) {
        runtime.ensure_every_sender_dsp(&mut state.lock(), &settings);
    }
    runtime.update_own_metadata();
}
