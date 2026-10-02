//! [`DebugWindow::spawn`], the [`DebugWindowHandle`] the plugin keeps, and the UI thread's loop.
//!
//! **Role:** runs the one long-lived UI thread per process and turns the plugin's show, hide,
//! publish and shutdown calls into windows opened and closed on that thread, so the single
//! event loop winit allows is reused for every window.
//!
//! **Position:** inside `window_control`; uses `window_state` and a `WindowBackend` (the eframe
//! one from `egui_window` in the plugin, a fake in tests); used by `tfar_plugin_1_0` from the
//! `/tfar debug` command and `ts3plugin_shutdown`.
//!
//! **Signals and state:** the thread `tfar-debug-window` waits on the shared condition variable
//! while hidden and runs the backend while shown; a process-wide flag records that the thread has
//! been spawned.
//!
//! **Invariants:** at most one UI thread per process, ever, as winit refuses a second event loop;
//! a window opens only on a show request and closes on hide, shutdown or the user's close button,
//! after which it stays closed until the next show; `shutdown` returns within one second.

#[cfg(test)]
#[path = "tests/debug_window_handle.rs"]
mod tests;

use std::ffi::OsStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use super::window_backend::WindowBackend;
use super::window_state::{SharedWindowState, WindowPhase, WindowRequest, WindowSession};
use crate::debug_snapshot::DebugSnapshot;
use crate::egui_window::EframeBackend;
use crate::{Error, Result};

/// How long `shutdown` waits for the UI thread to end.
pub const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(1);

/// The name of the UI thread, as shown in a debugger or `top -H`.
const UI_THREAD_NAME: &str = "tfar-debug-window";

/// Set once the UI thread of this process has been spawned.
static UI_THREAD_SPAWNED: AtomicBool = AtomicBool::new(false);

/// The debug window's entry point.
#[derive(Debug)]
pub struct DebugWindow;

impl DebugWindow {
    /// Starts the UI thread with no window open; [`DebugWindowHandle::show`] opens one titled
    /// `title`. Call it once per process and keep the handle until plugin shutdown.
    ///
    /// # Errors
    ///
    /// [`Error::NoDisplay`] when `DISPLAY` is unset or empty, so the plugin can print a message
    /// instead; [`Error::AlreadySpawned`] on a second call in the same process;
    /// [`Error::ThreadSpawn`] when the thread cannot start.
    pub fn spawn(title: &str) -> Result<DebugWindowHandle> {
        require_display(std::env::var_os("DISPLAY").as_deref())?;
        claim_process_slot(&UI_THREAD_SPAWNED)?;
        spawn_with_backend(title, EframeBackend).inspect_err(|_| {
            // No event loop was created, so a later spawn may still try.
            UI_THREAD_SPAWNED.store(false, Ordering::SeqCst);
        })
    }
}

/// The plugin's handle on the UI thread. Dropping it shuts the thread down as `shutdown` does.
pub struct DebugWindowHandle {
    state: Arc<SharedWindowState>,
    thread: Option<JoinHandle<()>>,
}

impl DebugWindowHandle {
    /// Opens the window, or keeps it open. It returns at once; the window appears on the UI
    /// thread.
    pub fn show(&self) {
        self.state.request(WindowRequest::Shown);
    }

    /// Closes the window, or keeps it closed; the UI thread stays, ready for the next show.
    pub fn hide(&self) {
        self.state.request(WindowRequest::Hidden);
    }

    /// Replaces what the window shows; an open window redraws, a closed one shows it when opened.
    pub fn publish(&self, snapshot: DebugSnapshot) {
        self.state.publish(snapshot);
    }

    /// Whether a window is open (or opening) now.
    pub fn is_open(&self) -> bool {
        self.state.phase() == WindowPhase::Open
    }

    /// Why the last window failed to open, once; `None` when it opened or was never asked to.
    pub fn take_window_error(&self) -> Option<String> {
        self.state.control.lock().last_error.take()
    }

    /// Closes any window, ends the UI thread and joins it.
    ///
    /// # Errors
    ///
    /// [`Error::ShutdownTimedOut`] when the thread has not ended within [`SHUTDOWN_TIMEOUT`]; it
    /// is then left to end on its own.
    pub fn shutdown(mut self) -> Result<()> {
        self.stop()
    }

    /// The shutdown both `shutdown` and `drop` run; a second call finds no thread and succeeds.
    fn stop(&mut self) -> Result<()> {
        let Some(thread) = self.thread.take() else {
            return Ok(());
        };
        self.state.request(WindowRequest::ShutDown);
        if !self
            .state
            .wait_for_phase(WindowPhase::Stopped, SHUTDOWN_TIMEOUT)
        {
            return Err(Error::ShutdownTimedOut {
                waited: SHUTDOWN_TIMEOUT,
            });
        }
        // The thread has passed its last statement, so the join returns at once.
        thread.join().map_err(|_| Error::UiThreadPanicked)
    }
}

impl Drop for DebugWindowHandle {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            tracing::warn!(%error, "the debug window did not shut down cleanly");
        }
    }
}

impl core::fmt::Debug for DebugWindowHandle {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("DebugWindowHandle")
            .field("phase", &self.state.phase())
            .finish_non_exhaustive()
    }
}

/// `DISPLAY` must name an X11 display: the window runs on X11 only.
pub(crate) fn require_display(display: Option<&OsStr>) -> Result<()> {
    match display {
        Some(value) if !value.is_empty() => Ok(()),
        _ => Err(Error::NoDisplay),
    }
}

/// Claims the one UI thread `flag` stands for.
pub(crate) fn claim_process_slot(flag: &AtomicBool) -> Result<()> {
    if flag.swap(true, Ordering::SeqCst) {
        Err(Error::AlreadySpawned)
    } else {
        Ok(())
    }
}

/// Starts the UI thread over `backend`, hidden.
pub(crate) fn spawn_with_backend(
    title: &str,
    backend: impl WindowBackend,
) -> Result<DebugWindowHandle> {
    let state = Arc::new(SharedWindowState::new());
    let thread_state = Arc::clone(&state);
    let title = title.to_owned();
    let thread = std::thread::Builder::new()
        .name(UI_THREAD_NAME.to_owned())
        .spawn(move || run_ui_thread(&thread_state, backend, &title))
        .map_err(|source| Error::ThreadSpawn { source })?;
    Ok(DebugWindowHandle {
        state,
        thread: Some(thread),
    })
}

/// The UI thread: wait while hidden, run one window while shown, end on shutdown.
fn run_ui_thread(state: &SharedWindowState, mut backend: impl WindowBackend, title: &str) {
    loop {
        {
            let mut control = state.control.lock();
            while control.request == WindowRequest::Hidden {
                state.changed.wait(&mut control);
            }
            if control.request == WindowRequest::ShutDown {
                break;
            }
            control.phase = WindowPhase::Open;
        }
        state.changed.notify_all();
        let result = backend.run_window(&WindowSession::new(state, title));
        state.clear_waker();
        {
            let mut control = state.control.lock();
            control.phase = WindowPhase::Hidden;
            // Closed by the user or failed: stay closed until the plugin asks again.
            if control.request == WindowRequest::Shown {
                control.request = WindowRequest::Hidden;
            }
            if let Err(error) = result {
                tracing::warn!(%error, "the debug window could not open");
                control.last_error = Some(error.to_string());
            }
        }
        state.changed.notify_all();
    }
    // The backend goes first, so whatever it owns is released before the handle sees `Stopped`.
    drop(backend);
    state.control.lock().phase = WindowPhase::Stopped;
    state.changed.notify_all();
}
