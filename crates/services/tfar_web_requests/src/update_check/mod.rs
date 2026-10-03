//! The update check: fetch the published version, compare it with the plugin's, call back when it
//! is newer.

mod latest_version;
mod version_comparison;

pub use latest_version::{
    UPDATE_AVAILABLE_MESSAGE, UPDATE_CHECK_TIMEOUT, UPDATE_CHECK_URL, fetch_latest_version,
    spawn_cancellable_update_check, spawn_update_check,
};
pub use version_comparison::{is_update_available, is_version_string};
