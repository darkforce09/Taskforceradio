//! Fetching the published version and the background update check the plugins start at load.
//!
//! **Role:** `fetch_latest_version` reads `current_version.txt`; `spawn_update_check` and
//! `spawn_cancellable_update_check` do that on their own thread and call back only when a newer
//! version is published, so the plugin can print [`UPDATE_AVAILABLE_MESSAGE`] (where the
//! originals showed a message box).
//!
//! **Position:** inside `update_check`; uses `http_client` and `version_comparison`; called by
//! the plugin apps from `ts3plugin_init`.
//!
//! **Signals and state:** one short-lived thread, `tfar-update-check`, per check; it holds the
//! callback until it returns, at most [`UPDATE_CHECK_TIMEOUT`] later. The cancellable check shares
//! a cancel flag with its [`BackgroundRequest`] handle.
//!
//! **Invariants:** the callback runs at most once, on the check thread, and only after a fetch
//! that succeeded with a strictly newer version, and never after the check was cancelled; every
//! failure is logged and swallowed.

#[cfg(test)]
#[path = "tests/latest_version.rs"]
mod tests;

use std::thread::JoinHandle;
use std::time::Duration;

use super::version_comparison::is_update_available;
use crate::background_request::{BackgroundRequest, CancelFlag};
use crate::http_client::{HttpClient, ProxyChoice};
use crate::{Error, Result};

/// Where the published version lives. The originals asked `raw.github.com` over plain HTTP,
/// which now redirects; this is the address it redirects to, and redirects are still followed.
pub const UPDATE_CHECK_URL: &str = "https://raw.githubusercontent.com/michail-nikolaev/task-force-arma-3-radio/master/current_version.txt";

/// How long the background check may take, connect to last byte.
pub const UPDATE_CHECK_TIMEOUT: Duration = Duration::from_secs(10);

/// What the plugin prints when a newer version is published, word for word the originals'
/// message-box text.
pub const UPDATE_AVAILABLE_MESSAGE: &str =
    "New version of Task Force Arrowhead Radio is available. Check radio.task-force.ru/en";

/// The name of the check thread, as shown in a debugger or `top -H`.
const UPDATE_CHECK_THREAD_NAME: &str = "tfar-update-check";

/// Fetches the version file at `url` within `timeout` and returns its body with surrounding
/// whitespace trimmed. The body is not checked here: [`is_update_available`] decides what counts.
///
/// # Errors
///
/// [`Error::TimedOut`] when the request outlasts `timeout`; [`Error::Request`] for any other
/// failure, a 4xx or 5xx status included.
pub fn fetch_latest_version(url: &str, timeout: Duration) -> Result<String> {
    fetch_latest_version_with(&HttpClient::new(timeout, ProxyChoice::FromEnvironment), url)
}

/// Starts the update check against [`UPDATE_CHECK_URL`] on its own thread, with
/// [`UPDATE_CHECK_TIMEOUT`], and calls `on_update` there when the published version is newer
/// than `current`. A failed fetch is logged at debug level and calls nothing.
///
/// The returned handle may be dropped to detach the check, or joined; the check cannot be
/// cancelled. A plugin that must stop and join it within a deadline at shutdown uses
/// [`spawn_cancellable_update_check`] instead.
///
/// # Errors
///
/// [`Error::ThreadSpawn`] when the thread cannot start.
pub fn spawn_update_check(
    current: &str,
    on_update: impl FnOnce() + Send + 'static,
) -> Result<JoinHandle<()>> {
    spawn_update_check_with(
        HttpClient::new(UPDATE_CHECK_TIMEOUT, ProxyChoice::FromEnvironment),
        UPDATE_CHECK_URL.to_owned(),
        current,
        on_update,
    )
}

/// [`fetch_latest_version`] with a given client.
pub(crate) fn fetch_latest_version_with(client: &HttpClient, url: &str) -> Result<String> {
    client.get_text(url).map(|body| body.trim().to_owned())
}

/// [`spawn_update_check`] with a given client and URL.
pub(crate) fn spawn_update_check_with(
    client: HttpClient,
    url: String,
    current: &str,
    on_update: impl FnOnce() + Send + 'static,
) -> Result<JoinHandle<()>> {
    let current = current.to_owned();
    std::thread::Builder::new()
        .name(UPDATE_CHECK_THREAD_NAME.to_owned())
        .spawn(move || match fetch_latest_version_with(&client, &url) {
            Ok(latest) if is_update_available(&current, &latest) => {
                tracing::info!(%current, %latest, "a newer TFAR version is published");
                on_update();
            }
            Ok(latest) => tracing::debug!(%current, %latest, "no newer TFAR version is published"),
            Err(error) => tracing::debug!(%error, "the update check failed"),
        })
        .map_err(|source| Error::ThreadSpawn {
            thread: UPDATE_CHECK_THREAD_NAME,
            source,
        })
}

/// Starts the update check of [`spawn_update_check`] as a [`BackgroundRequest`] the plugin can
/// cancel and join at shutdown: against [`UPDATE_CHECK_URL`], with [`UPDATE_CHECK_TIMEOUT`],
/// calling `on_update` on the check thread when the published version is newer than `current`.
///
/// The worker checks the cancel flag before the fetch and after it, so a check cancelled in time
/// sends nothing, and a cancelled check never calls `on_update`, even when the answer arrives.
///
/// # Errors
///
/// [`Error::ThreadSpawn`] when the thread cannot start.
pub fn spawn_cancellable_update_check(
    current: &str,
    on_update: impl FnOnce() + Send + 'static,
) -> Result<BackgroundRequest> {
    spawn_cancellable_update_check_with(
        HttpClient::new(UPDATE_CHECK_TIMEOUT, ProxyChoice::FromEnvironment),
        UPDATE_CHECK_URL.to_owned(),
        current,
        on_update,
        CancelFlag::new(),
    )
}

/// [`spawn_cancellable_update_check`] with a given client, URL and cancel flag.
pub(crate) fn spawn_cancellable_update_check_with(
    client: HttpClient,
    url: String,
    current: &str,
    on_update: impl FnOnce() + Send + 'static,
    cancel: CancelFlag,
) -> Result<BackgroundRequest> {
    let current = current.to_owned();
    BackgroundRequest::spawn(UPDATE_CHECK_THREAD_NAME, cancel, move |cancel| {
        if cancel.is_cancelled() {
            tracing::debug!("the update check was cancelled before it started");
            return;
        }
        let fetched = fetch_latest_version_with(&client, &url);
        if cancel.is_cancelled() {
            tracing::debug!("the update check was cancelled; its answer is dropped");
            return;
        }
        match fetched {
            Ok(latest) if is_update_available(&current, &latest) => {
                tracing::info!(%current, %latest, "a newer TFAR version is published");
                on_update();
            }
            Ok(latest) => tracing::debug!(%current, %latest, "no newer TFAR version is published"),
            Err(error) => tracing::debug!(%error, "the update check failed"),
        }
    })
}
