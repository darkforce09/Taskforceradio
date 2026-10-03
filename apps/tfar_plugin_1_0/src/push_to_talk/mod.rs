//! Push-to-talk for radio tangents: the microphone forced on while a tangent is held, voice
//! activation saved and restored, the delayed release, and the broadcast to the channel.
//!
//! **Role:** the tangent part of `CommandProcessor` (`reference/tfar_1_0/ts/src/
//! CommandProcessor.cpp:293-386`, `:629-648`): `tangentCriticalSection`, `vadEnabled`,
//! `waitingForTangentOff`, `skipTangentOff`, `process_tangent_off` and
//! `disableVoiceAndSendCommand`, with one release thread instead of a detached thread per release.
//!
//! **Position:** owned by `PluginRuntime`; pressed from the game link worker, released through
//! the thread `tfar-tangent-release`, restored at game end.
//!
//! **Signals and state:** the [`TangentGate`] behind a `parking_lot` mutex shared with the release
//! thread, which waits on a job channel and a stop channel; TeamSpeak is called with the gate
//! held, which no audio callback and no TeamSpeak callback takes.
//!
//! **Invariants:** a release runs `push_to_talk_delay + tangentReleaseDelay` after the tangent is
//! let go, unless a press came in between; the release thread is joined in `stop`.

pub mod tangent_gate;

use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use parking_lot::Mutex;
use tfar_ts3_host::{PluginCommandTarget, ServerConnectionHandlerId};

pub use tangent_gate::TangentGate;

use crate::teamspeak_actions::TeamSpeakActions;

/// The name of the release thread.
pub const RELEASE_THREAD_NAME: &str = "tfar-tangent-release";

/// One delayed release.
struct ReleaseJob {
    /// TeamSpeak.
    ts: &'static dyn TeamSpeakActions,
    /// The connection the tangent was pressed on.
    server: ServerConnectionHandlerId,
    /// The plugin command telling the channel the tangent is released.
    broadcast: String,
    /// How long to wait before releasing.
    delay: Duration,
}

/// The running release thread.
struct ReleaseWorker {
    /// Hands it releases.
    jobs: Sender<ReleaseJob>,
    /// Dropped to stop it at once, even mid-delay.
    stop: Sender<()>,
    /// Joined in [`PushToTalk::stop`].
    thread: JoinHandle<()>,
}

/// The push-to-talk state; see the module header.
#[derive(Default)]
pub struct PushToTalk {
    gate: Arc<Mutex<TangentGate>>,
    worker: Mutex<Option<ReleaseWorker>>,
}

impl core::fmt::Debug for PushToTalk {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PushToTalk")
            .field("gate", &*self.gate.lock())
            .finish_non_exhaustive()
    }
}

/// Switches the microphone and sends the tangent broadcast (`disableVoiceAndSendCommand`).
fn switch_voice_and_broadcast(
    ts: &dyn TeamSpeakActions,
    server: ServerConnectionHandlerId,
    broadcast: &str,
    is_pressed: bool,
    was_voice_activation_enabled: bool,
) {
    ts.set_input_deactivated(server, !(is_pressed || was_voice_activation_enabled));
    ts.send_plugin_command(server, broadcast, PluginCommandTarget::CurrentChannel);
}

impl PushToTalk {
    /// Nothing pressed, no release thread yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// A tangent is pressed: unless a release is pending, voice activation is saved and switched
    /// off, the microphone forced on and `broadcast` sent; with a release pending, that release
    /// is skipped instead.
    pub fn press(
        &self,
        ts: &dyn TeamSpeakActions,
        server: ServerConnectionHandlerId,
        broadcast: &str,
    ) {
        let mut gate = self.gate.lock();
        if !gate.press() {
            return;
        }
        gate.was_voice_activation_enabled = ts.is_voice_activation_enabled(server);
        if gate.was_voice_activation_enabled {
            ts.set_voice_activation(server, false);
        }
        switch_voice_and_broadcast(
            ts,
            server,
            broadcast,
            true,
            gate.was_voice_activation_enabled,
        );
    }

    /// A tangent is let go: after `delay` the microphone and voice activation return to what they
    /// were and `broadcast` is sent, unless a press came in between.
    pub fn release(
        &self,
        ts: &'static dyn TeamSpeakActions,
        server: ServerConnectionHandlerId,
        broadcast: String,
        delay: Duration,
    ) {
        self.gate.lock().begin_release();
        let job = ReleaseJob {
            ts,
            server,
            broadcast,
            delay,
        };
        let mut worker = self.worker.lock();
        if worker.is_none() {
            match spawn_release_worker(Arc::clone(&self.gate)) {
                Ok(started) => *worker = Some(started),
                Err(error) => {
                    tracing::error!(%error, "the tangent release thread did not start");
                    drop(worker);
                    run_release(&self.gate, &job);
                    return;
                }
            }
        }
        if let Some(started) = worker.as_ref()
            && let Err(refused) = started.jobs.send(job)
        {
            drop(worker);
            run_release(&self.gate, &refused.0);
        }
    }

    /// The game ended: the microphone and voice activation return to what they were before the
    /// last transmission (`onGameEnd`).
    pub fn restore_on_game_end(
        &self,
        ts: &dyn TeamSpeakActions,
        server: ServerConnectionHandlerId,
    ) {
        let gate = self.gate.lock();
        ts.set_input_deactivated(server, !gate.was_voice_activation_enabled);
        if gate.was_voice_activation_enabled {
            ts.set_voice_activation(server, true);
        }
    }

    /// Stops the release thread, dropping releases still waiting, and joins it.
    pub fn stop(&self) {
        let worker = self.worker.lock().take();
        if let Some(ReleaseWorker { jobs, stop, thread }) = worker {
            drop(stop);
            drop(jobs);
            if thread.join().is_err() {
                tracing::error!(thread = RELEASE_THREAD_NAME, "a thread panicked");
            }
        }
    }
}

/// The release itself, under the gate (`process_tangent_off` after its sleep).
fn run_release(gate: &Mutex<TangentGate>, job: &ReleaseJob) {
    let mut gate = gate.lock();
    if !gate.finish_release() {
        return;
    }
    if gate.was_voice_activation_enabled {
        job.ts.set_voice_activation(job.server, true);
    }
    switch_voice_and_broadcast(
        job.ts,
        job.server,
        &job.broadcast,
        false,
        gate.was_voice_activation_enabled,
    );
}

/// Starts the release thread over `gate`.
fn spawn_release_worker(gate: Arc<Mutex<TangentGate>>) -> std::io::Result<ReleaseWorker> {
    let (jobs, job_receiver) = crossbeam_channel::unbounded::<ReleaseJob>();
    let (stop, stop_receiver) = crossbeam_channel::bounded::<()>(0);
    let thread = std::thread::Builder::new()
        .name(RELEASE_THREAD_NAME.to_owned())
        .spawn(move || release_loop(&gate, &job_receiver, &stop_receiver))?;
    Ok(ReleaseWorker { jobs, stop, thread })
}

/// The release thread: each job waits its delay, cut short only by the stop signal.
fn release_loop(gate: &Mutex<TangentGate>, jobs: &Receiver<ReleaseJob>, stop: &Receiver<()>) {
    loop {
        crossbeam_channel::select! {
            recv(jobs) -> job => {
                let Ok(job) = job else { return };
                match stop.recv_timeout(job.delay) {
                    Err(RecvTimeoutError::Timeout) => run_release(gate, &job),
                    Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
                }
            }
            recv(stop) -> _ => return,
        }
    }
}
