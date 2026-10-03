//! [`BackgroundRequests`]: the update check and the Piwik tracking requests the plugin started,
//! kept so shutdown can cancel them and join them within its budget.
//!
//! **Role:** hold every `tfar_web_requests::BackgroundRequest` handle from start to shutdown;
//! forget a request once its thread has ended; at shutdown cancel them all at once and join them
//! against one deadline.
//!
//! **Position:** owned by `PluginRuntime`; filled by `ts3plugin_init` (the update check) and the
//! async game message path (`TRACK`), drained by `PluginRuntime::shut_down`.
//!
//! **Signals and state:** the handles behind one `parking_lot` mutex, a leaf lock never held
//! while another lock is taken.
//!
//! **Invariants:** a request in flight is not interrupted (its `ureq` timeout ends it: 10 s for
//! the update check, 5 s for a tracking request), so a request still running at the deadline is
//! logged and left; its handle was cancelled, so it acts on nothing when its answer arrives.

use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tfar_web_requests::prelude::BackgroundRequest;

/// The web requests the plugin started; see the module header.
#[derive(Debug, Default)]
pub struct BackgroundRequests {
    requests: Mutex<Vec<BackgroundRequest>>,
}

/// What [`BackgroundRequests::cancel_and_join`] found.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JoinReport {
    /// The requests whose thread ended and was joined.
    pub joined: usize,
    /// The requests still running at the deadline, left to end within their timeout.
    pub left_running: usize,
}

impl BackgroundRequests {
    /// No request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Keeps `request` until shutdown, forgetting (and joining) the requests that have ended.
    pub fn keep(&self, request: BackgroundRequest) {
        let mut requests = self.requests.lock();
        requests.retain_mut(|kept| !(kept.is_finished() && kept.join_timeout(Duration::ZERO)));
        requests.push(request);
    }

    /// The requests kept now, finished or not.
    pub fn len(&self) -> usize {
        self.requests.lock().len()
    }

    /// True when no request is kept.
    pub fn is_empty(&self) -> bool {
        self.requests.lock().is_empty()
    }

    /// Cancels every request without waiting, so none starts or acts from now on.
    pub fn cancel_all(&self) {
        for request in self.requests.lock().iter() {
            request.cancel();
        }
    }

    /// Cancels every request and joins each one until `budget` from now has passed; a request
    /// still running then is logged and left.
    pub fn cancel_and_join(&self, budget: Duration) -> JoinReport {
        let mut requests = std::mem::take(&mut *self.requests.lock());
        for request in &requests {
            request.cancel();
        }
        let deadline = Instant::now() + budget;
        let mut report = JoinReport::default();
        for request in &mut requests {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if request.join_timeout(remaining) {
                report.joined += 1;
            } else {
                report.left_running += 1;
            }
        }
        if report.left_running > 0 {
            tracing::warn!(
                left_running = report.left_running,
                budget_ms = budget.as_millis(),
                "web requests are still waiting for the network after the shutdown budget; \
                 they are cancelled and end within their timeout"
            );
        }
        report
    }
}
