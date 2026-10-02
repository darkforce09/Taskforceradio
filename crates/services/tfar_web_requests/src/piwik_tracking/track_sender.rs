//! Sending a tracking request: fire and forget on a short-lived thread.
//!
//! The Piwik hosts may be gone; a request that fails is logged at debug level and nothing else
//! happens, as the originals ignored every result.

#[cfg(test)]
#[path = "tests/track_sender.rs"]
mod tests;

use std::thread::JoinHandle;
use std::time::Duration;

use url::Url;

use crate::http_client::{HttpClient, ProxyChoice};
use crate::{Error, Result};

/// How long one tracking request may take, connect to last byte.
pub const TRACK_TIMEOUT: Duration = Duration::from_secs(5);

/// The name of the tracking thread, as shown in a debugger or `top -H`.
const TRACK_THREAD_NAME: &str = "tfar-piwik-track";

/// Sends `GET url` on its own thread with [`TRACK_TIMEOUT`], reads and discards the answer and
/// logs a failure; the caller never waits.
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
