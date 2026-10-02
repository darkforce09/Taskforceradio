//! Piwik tracking: the 0.9 and 1.0 `TRACK` requests and the fire-and-forget sender.

mod custom_variables;
mod track_request_0_9;
mod track_request_1_0;
mod track_sender;

pub use custom_variables::{CustomVariable, parse_custom_variables};
pub use track_request_0_9::{PIWIK_HOST, build_track_request_0_9};
pub use track_request_1_0::{PIWIK_BETA_HOST, build_track_request_1_0};
pub use track_sender::{TRACK_TIMEOUT, spawn_track};
