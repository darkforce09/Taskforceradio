//! The state the handle and the UI thread share: what the plugin asks for, what the UI thread is
//! doing, the published snapshot and the waker of an open window.
//!
//! **Role:** the one meeting point between the plugin's threads (show, hide, publish, shutdown)
//! and the UI thread; [`WindowSession`] is the part a window backend sees.
//!
//! **Position:** inside `window_control`; created by `debug_window_handle`, read by every
//! `WindowBackend`.
//!
//! **Signals and state:** `control` (a `parking_lot` mutex with a condition variable) holds the
//! request and the phase; `snapshot` and `waker` are `arc-swap` cells, so publishing and waking
//! take no lock. The plugin's threads write requests and snapshots; the UI thread writes the phase
//! and the waker.
//!
//! **Invariants:** a request only moves forward to `ShutDown`, never back; no lock is held while
//! the waker runs, so a backend's repaint call cannot deadlock against a request.

use std::sync::Arc;
use std::time::{Duration, Instant};

use arc_swap::{ArcSwap, ArcSwapOption};
use parking_lot::{Condvar, Mutex};

use crate::debug_snapshot::DebugSnapshot;

/// What the plugin last asked of the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WindowRequest {
    /// No window: the UI thread waits.
    Hidden,
    /// A window on screen.
    Shown,
    /// The UI thread ends; nothing reopens it.
    ShutDown,
}

/// What the UI thread is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WindowPhase {
    /// Waiting for a show request.
    Hidden,
    /// A window is opening, open or closing.
    Open,
    /// The thread has ended.
    Stopped,
}

/// The guarded part of the shared state.
#[derive(Debug)]
pub(crate) struct WindowControl {
    pub(crate) request: WindowRequest,
    pub(crate) phase: WindowPhase,
    /// Why the last window failed to open, until the plugin takes it.
    pub(crate) last_error: Option<String>,
}

/// How a backend wakes its open window from another thread (eframe: `request_repaint`).
pub(crate) struct Waker(Box<dyn Fn() + Send + Sync>);

impl Waker {
    /// Wraps the wake call.
    pub(crate) fn new(wake: impl Fn() + Send + Sync + 'static) -> Self {
        Self(Box::new(wake))
    }
}

/// Shared between the handle and the UI thread.
pub(crate) struct SharedWindowState {
    pub(crate) control: Mutex<WindowControl>,
    pub(crate) changed: Condvar,
    snapshot: ArcSwap<DebugSnapshot>,
    waker: ArcSwapOption<Waker>,
}

impl SharedWindowState {
    /// Hidden, with an empty snapshot and no waker.
    pub(crate) fn new() -> Self {
        Self {
            control: Mutex::new(WindowControl {
                request: WindowRequest::Hidden,
                phase: WindowPhase::Hidden,
                last_error: None,
            }),
            changed: Condvar::new(),
            snapshot: ArcSwap::from_pointee(DebugSnapshot::new()),
            waker: ArcSwapOption::empty(),
        }
    }

    /// Sets the request, unless the thread is shutting down, and wakes everyone who waits on it.
    pub(crate) fn request(&self, request: WindowRequest) {
        {
            let mut control = self.control.lock();
            if control.request != WindowRequest::ShutDown {
                control.request = request;
            }
        }
        self.changed.notify_all();
        self.wake_window();
    }

    /// Replaces the published snapshot and wakes an open window to draw it.
    pub(crate) fn publish(&self, snapshot: DebugSnapshot) {
        self.snapshot.store(Arc::new(snapshot));
        self.wake_window();
    }

    /// The latest published snapshot.
    pub(crate) fn snapshot(&self) -> Arc<DebugSnapshot> {
        self.snapshot.load_full()
    }

    /// The UI thread's phase now.
    pub(crate) fn phase(&self) -> WindowPhase {
        self.control.lock().phase
    }

    /// Waits until the phase is `phase` or `timeout` passes; whether it got there.
    pub(crate) fn wait_for_phase(&self, phase: WindowPhase, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let mut control = self.control.lock();
        while control.phase != phase {
            if self.changed.wait_until(&mut control, deadline).timed_out() {
                return control.phase == phase;
            }
        }
        true
    }

    /// Calls the open window's waker, if one is registered; no lock is held meanwhile.
    fn wake_window(&self) {
        if let Some(waker) = self.waker.load_full() {
            (waker.0)();
        }
    }

    /// Registers the open window's waker.
    fn set_waker(&self, waker: Waker) {
        self.waker.store(Some(Arc::new(waker)));
    }

    /// Forgets the waker of a window that has closed.
    pub(crate) fn clear_waker(&self) {
        self.waker.store(None);
    }
}

/// What a window backend sees of the shared state while one window is open.
pub(crate) struct WindowSession<'state> {
    state: &'state SharedWindowState,
    title: &'state str,
}

impl<'state> WindowSession<'state> {
    /// A session over `state` for a window titled `title`.
    pub(crate) fn new(state: &'state SharedWindowState, title: &'state str) -> Self {
        Self { state, title }
    }

    /// The window title.
    pub(crate) fn title(&self) -> &str {
        self.title
    }

    /// The latest published snapshot.
    pub(crate) fn snapshot(&self) -> Arc<DebugSnapshot> {
        self.state.snapshot()
    }

    /// Whether the window should close now: the plugin asked to hide it or to shut down.
    pub(crate) fn should_close(&self) -> bool {
        self.state.control.lock().request != WindowRequest::Shown
    }

    /// Registers how to wake this window from another thread; the plugin's show, hide, publish and
    /// shutdown calls run it.
    pub(crate) fn set_waker(&self, waker: Waker) {
        self.state.set_waker(waker);
    }
}
