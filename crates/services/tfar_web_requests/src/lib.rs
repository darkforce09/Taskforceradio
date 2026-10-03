//! The update check and the Piwik tracking calls, over blocking HTTP and HTTPS.
//!
//! **Role:** the plugins' outbound HTTP: the update check against the published version and the
//! Piwik `TRACK` calls, over `ureq` with rustls, each on a short-lived worker thread, detached or
//! behind a [`BackgroundRequest`] handle the plugin cancels and joins at shutdown.
//!
//! **Position:** `crates/services`, tier 0, depending on no workspace crate; used by
//! `tfar_plugin_0_9`, `tfar_plugin_1_0`.
//!
//! **Signals and state:** no shared state between requests; each `spawn_*` function starts one
//! named thread that ends within its timeout (10 s and 5 s); a cancellable request shares one
//! cancel flag with its handle.
//!
//! **Invariants:** no request runs on a TeamSpeak audio callback or blocks the caller of a
//! `spawn_*` function; a failed request is logged and never reaches the game; a cancelled request
//! that has not started is never sent and a cancelled check never calls back.

mod background_request;
mod crate_role;
mod error;
mod http_client;
mod piwik_tracking;
pub mod prelude;
mod update_check;

#[cfg(test)]
#[path = "tests/local_responder.rs"]
mod local_responder;

pub use background_request::BackgroundRequest;
pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
pub use piwik_tracking::{
    CustomVariable, PIWIK_BETA_HOST, PIWIK_HOST, TRACK_TIMEOUT, build_track_request_0_9,
    build_track_request_1_0, parse_custom_variables, spawn_cancellable_track, spawn_track,
};
pub use update_check::{
    UPDATE_AVAILABLE_MESSAGE, UPDATE_CHECK_TIMEOUT, UPDATE_CHECK_URL, fetch_latest_version,
    is_update_available, is_version_string, spawn_cancellable_update_check, spawn_update_check,
};
pub use url::Url;
