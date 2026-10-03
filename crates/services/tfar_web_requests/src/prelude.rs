//! The names a caller of `tfar_web_requests` imports with `use tfar_web_requests::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::background_request::BackgroundRequest;
pub use crate::crate_role::CRATE_ROLE;
pub use crate::piwik_tracking::{
    build_track_request_0_9, build_track_request_1_0, spawn_cancellable_track, spawn_track,
};
pub use crate::update_check::{
    UPDATE_AVAILABLE_MESSAGE, fetch_latest_version, is_update_available,
    spawn_cancellable_update_check, spawn_update_check,
};
