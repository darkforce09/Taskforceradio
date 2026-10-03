//! [`BackgroundRequest`]: the handle of one request running on its own thread, which the plugin can
//! cancel and join with a deadline at shutdown.
//!
//! **Role:** starts the worker thread of a cancellable update check or tracking request, shares a
//! cancel flag with it, and lets the owner cancel, wait for the thread with a timeout, or drop the
//! handle without waiting.
//!
//! **Position:** inside `tfar_web_requests`; used by `update_check` and `piwik_tracking`, whose
//! `spawn_cancellable_*` functions hand the handle to the plugin apps.
//!
//! **Signals and state:** one `AtomicBool` cancel flag shared by the handle and the worker; a
//! `std::sync::mpsc` channel whose sending side lives on the worker and disconnects when it ends,
//! so the handle can wait for the end with a timeout.
//!
//! **Invariants:** the worker checks the flag before its request and again after it, so a request
//! cancelled in time never starts and a cancelled request's result is never acted on; the request
//! itself ends within its `ureq` timeout; dropping the handle cancels and never blocks.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::{Error, Result};

#[cfg(test)]
#[path = "tests/background_request.rs"]
mod tests;

/// The cancel flag a worker reads; cloned into the worker thread.
#[derive(Clone, Debug, Default)]
pub(crate) struct CancelFlag(Arc<AtomicBool>);

impl CancelFlag {
    /// A flag that is not set.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// True once the owner cancelled the request.
    pub(crate) fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    /// Sets the flag; the worker sees it at its next check.
    pub(crate) fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

/// One request running on its own named thread.
///
/// [`cancel`](Self::cancel) asks the worker to stop: a request that has not started yet is never
/// sent, and the result of one in flight is dropped (no update callback runs). A request in
/// flight is not interrupted; it ends within its timeout (10 s for the update check, 5 s for a
/// tracking request). [`join_timeout`](Self::join_timeout) waits for the worker up to a
/// deadline, so plugin shutdown never waits longer than its budget. Dropping the handle cancels
/// the request and returns at once, leaving the worker to end on its own.
#[derive(Debug)]
pub struct BackgroundRequest {
    /// The flag shared with the worker.
    cancel: CancelFlag,
    /// Disconnects when the worker ends.
    finished: Receiver<()>,
    /// The worker, until it is joined.
    thread: Option<JoinHandle<()>>,
}

impl BackgroundRequest {
    /// Starts `work` on a thread called `thread_name`, handing it the cancel flag.
    ///
    /// # Errors
    ///
    /// [`Error::ThreadSpawn`] when the thread cannot start.
    pub(crate) fn spawn(
        thread_name: &'static str,
        cancel: CancelFlag,
        work: impl FnOnce(&CancelFlag) + Send + 'static,
    ) -> Result<Self> {
        let (finished_sender, finished) = mpsc::channel::<()>();
        let worker_cancel = cancel.clone();
        let thread = std::thread::Builder::new()
            .name(thread_name.to_owned())
            .spawn(move || {
                // Dropped when the worker returns, which disconnects `finished`.
                let _finished_sender = finished_sender;
                work(&worker_cancel);
            })
            .map_err(|source| Error::ThreadSpawn {
                thread: thread_name,
                source,
            })?;
        Ok(Self {
            cancel,
            finished,
            thread: Some(thread),
        })
    }

    /// Asks the worker to stop; returns at once. Calling it again changes nothing.
    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    /// True once [`cancel`](Self::cancel) was called.
    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    /// True once the worker has ended (or was joined).
    pub fn is_finished(&self) -> bool {
        self.thread.as_ref().is_none_or(JoinHandle::is_finished)
    }

    /// Waits up to `timeout` for the worker to end and joins it; true when it ended (at once if
    /// it already had), false when it is still running at the deadline. Does not cancel: call
    /// [`cancel`](Self::cancel) first to make the worker stop early.
    pub fn join_timeout(&mut self, timeout: Duration) -> bool {
        let Some(thread) = self.thread.take() else {
            return true;
        };
        match self.finished.recv_timeout(timeout) {
            Ok(()) | Err(RecvTimeoutError::Disconnected) => {
                if thread.join().is_err() {
                    tracing::error!("a web request thread panicked");
                }
                true
            }
            Err(RecvTimeoutError::Timeout) => {
                self.thread = Some(thread);
                false
            }
        }
    }
}

impl Drop for BackgroundRequest {
    fn drop(&mut self) {
        self.cancel();
    }
}
