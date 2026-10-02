//! The update check and the Piwik tracking calls, over blocking HTTPS.
//!
//! **Role:** the plugins' outbound HTTP: the update check against the published version and the
//! Piwik `TRACK` calls, over `ureq` with rustls, off the audio thread.
//!
//! **Position:** `crates/services`, tier 0, depending on no workspace crate; used by
//! `tfar_plugin_0_9`, `tfar_plugin_1_0`.
//!
//! **Signals and state:** none; each request is one blocking call on a worker thread.
//!
//! **Invariants:** no request runs on a TeamSpeak audio callback.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
