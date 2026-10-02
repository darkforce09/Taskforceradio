//! The names a caller of `tfar_ts3_settings_db` imports with `use tfar_ts3_settings_db::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
pub use crate::push_to_talk_delay::PushToTalkDelay;
pub use crate::settings_database::{
    SETTINGS_DATABASE_FILE_NAME, read_push_to_talk_delay, settings_database_path,
};
