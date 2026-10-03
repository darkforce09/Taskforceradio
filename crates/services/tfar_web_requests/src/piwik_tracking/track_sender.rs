//! Sending a tracking request: fire and forget on a short-lived thread, detached or cancellable.
//!
//! **Role:** [`spawn_track`] sends one `GET` on a detached thread; [`spawn_cancellable_track`]
//! sends it behind a [`BackgroundRequest`], so the plugin can cancel and join the request at
//! shutdown.
//!
//! **Position:** inside `piwik_tracking`; uses `http_client` and `background_request`; called by
//! the plugin apps' `TRACK` command handlers with a URL from `track_request_0_9` or
//! `track_request_1_0`.
//!
//! **Signals and state:** one short-lived `tfar-piwik-track` thread per request, ending within
//! [`TRACK_TIMEOUT`]; a cancellable request shares one cancel flag with its handle.
//!
//! **Invariants:** the Piwik hosts may be gone, so a request that fails is logged at debug level
//! and nothing else happens, as the originals ignored every result; a cancellable request
//! cancelled before it starts is never sent.

#[cfg(test)]
#[path = "tests/track_sender.rs"]
mod tests;

use std::thread::JoinHandle;
use std::time::Duration;

use url::Url;

use crate::background_request::{BackgroundRequest, CancelFlag};
use crate::http_client::{HttpClient, ProxyChoice};
use crate::{Error, Result};

/// How long one tracking request may take, connect to last byte.
pub const TRACK_TIMEOUT: Duration = Duration::from_secs(5);

/// The name of the tracking thread, as shown in a debugger or `top -H`.
const TRACK_THREAD_NAME: &str = "tfar-piwik-track";

/// Sends `GET url` on its own thread with [`TRACK_TIMEOUT`], reads and discards the answer and
/// logs a failure; the caller never waits. The thread is detached and cannot be cancelled; a
/// plugin that must join it at shutdown uses [`spawn_cancellable_track`] instead.
///
/// # Errors
///
/// [`Error::ThreadSpawn`] when the thread cannot start.
pub fn spawn_track(url: Url) -> Result<()> {
    spawn_track_with(
        HttpClient::new(TRACK_TIMEOUT, ProxyChoice::FromEnvironment),
        url,
    )
    .map(drop)
}

/// Sends `GET url` as [`spawn_track`] does, as a [`BackgroundRequest`] the plugin can cancel and
/// join at shutdown. The worker checks the cancel flag before the request, so a request cancelled
/// in time is never sent, and after it, so a cancelled request logs nothing.
///
/// Keep the handle until the request is joined: dropping it cancels the request.
///
/// # Errors
///
/// [`Error::ThreadSpawn`] when the thread cannot start.
pub fn spawn_cancellable_track(url: Url) -> Result<BackgroundRequest> {
    spawn_cancellable_track_with(
        HttpClient::new(TRACK_TIMEOUT, ProxyChoice::FromEnvironment),
        url,
        CancelFlag::new(),
    )
}

/// [`spawn_track`] with a given client, returning the thread so a test can join it.
pub(crate) fn spawn_track_with(client: HttpClient, url: Url) -> Result<JoinHandle<()>> {
    std::thread::Builder::new()
        .name(TRACK_THREAD_NAME.to_owned())
        .spawn(move || {
            if let Err(error) = client.get_and_discard(url.as_str()) {
                tracing::debug!(%error, "a Piwik tracking request failed");
            }
        })
        .map_err(|source| Error::ThreadSpawn {
            thread: TRACK_THREAD_NAME,
            source,
        })
}

/// [`spawn_cancellable_track`] with a given client and cancel flag.
pub(crate) fn spawn_cancellable_track_with(
    client: HttpClient,
    url: Url,
    cancel: CancelFlag,
) -> Result<BackgroundRequest> {
    BackgroundRequest::spawn(TRACK_THREAD_NAME, cancel, move |cancel| {
        if cancel.is_cancelled() {
            tracing::debug!("a Piwik tracking request was cancelled before it was sent");
            return;
        }
        let sent = client.get_and_discard(url.as_str());
        if cancel.is_cancelled() {
            return;
        }
        if let Err(error) = sent {
            tracing::debug!(%error, "a Piwik tracking request failed");
        }
    })
}
