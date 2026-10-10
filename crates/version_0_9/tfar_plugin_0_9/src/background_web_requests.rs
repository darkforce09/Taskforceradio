//! `BackgroundWebRequests`: the update check and the Piwik requests the plugin has running, kept
//! so that shutdown can cancel and join them within its budget.
//!
//! **Role:** hold every `tfar_web_requests::BackgroundRequest` the plugin started (the update
//! check at init, one tracking request per `TRACK` game message), drop the finished ones as new
//! ones arrive, and at shutdown cancel them all, join them up to a deadline and log the ones
//! still in flight.
//!
//! **Position:** owned by `PluginCore`; the game link thread adds tracking requests,
//! `RunningPlugin` adds the update check, and in `ts3plugin_shutdown` calls
//! [`BackgroundWebRequests::cancel_all`] first and [`BackgroundWebRequests::cancel_and_join`]
//! last.
//!
//! **Signals and state:** the handles and a closed flag behind one `parking_lot::Mutex`, never
//! held while joining.
//!
//! **Invariants:** a request added after the close is cancelled at once; shutdown never waits
//! past its deadline: a request whose `ureq` call is in flight cannot be interrupted, so it is
//! cancelled (its answer is dropped), logged and left to end within its own timeout.

use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tfar_web_requests::BackgroundRequest;

#[cfg(test)]
#[path = "tests/background_web_requests.rs"]
mod tests;

/// The handles and whether shutdown has closed the set.
#[derive(Debug, Default)]
struct Requests {
    running: Vec<BackgroundRequest>,
    is_closed: bool,
}

/// The plugin's running web requests; see the module header.
#[derive(Debug, Default)]
pub struct BackgroundWebRequests {
    requests: Mutex<Requests>,
}

impl BackgroundWebRequests {
    /// An empty, open set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Keeps `request` until shutdown, first joining the requests that have finished; after
    /// [`cancel_and_join`](Self::cancel_and_join) the request is cancelled and dropped instead.
    pub fn keep(&self, request: BackgroundRequest) {
        let mut requests = self.requests.lock();
        if requests.is_closed {
            request.cancel();
            return;
        }
        requests
            .running
            .retain_mut(|running| !(running.is_finished() && running.join_timeout(Duration::ZERO)));
        requests.running.push(request);
    }

    /// How many kept requests have not been joined yet.
    pub fn len(&self) -> usize {
        self.requests.lock().running.len()
    }

    /// Whether no kept request is left to join.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Closes the set and cancels every kept request; returns at once. A request added later is
    /// cancelled as it arrives.
    pub fn cancel_all(&self) {
        let mut requests = self.requests.lock();
        requests.is_closed = true;
        requests.running.iter().for_each(BackgroundRequest::cancel);
    }

    /// [`cancel_all`](Self::cancel_all), then joins each kept request until `deadline`; returns
    /// how many were still running at the deadline, which are logged and left to end within
    /// their own timeout.
    pub fn cancel_and_join(&self, deadline: Instant) -> usize {
        self.cancel_all();
        let mut running = std::mem::take(&mut self.requests.lock().running);
        running.retain_mut(|request| {
            !request.join_timeout(deadline.saturating_duration_since(Instant::now()))
        });
        let still_running = running.len();
        if still_running > 0 {
            tracing::warn!(
                still_running,
                "web requests still wait for the network at shutdown; they are cancelled and end \
                 within their own timeout"
            );
        }
        still_running
    }
}
