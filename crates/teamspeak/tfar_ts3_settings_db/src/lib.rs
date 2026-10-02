//! Reads TeamSpeak's settings.db for the values the plugins need.
//!
//! **Role:** the read-only reader of TeamSpeak's `settings.db` SQLite file: the push-to-talk
//! delay of the default capture profile ([`read_push_to_talk_delay`], [`PushToTalkDelay`]).
//!
//! **Position:** `crates/teamspeak`, tier 0, depending on no workspace crate; used by
//! `tfar_plugin_0_9`, `tfar_plugin_1_0`.
//!
//! **Signals and state:** one read-only connection per read, closed before it returns.
//!
//! **Invariants:** the database is never written; every failure is a typed [`Error`] the
//! plugins read as "no delay".

mod crate_role;
mod error;
pub mod prelude;
mod push_to_talk_delay;
mod settings_database;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
pub use push_to_talk_delay::PushToTalkDelay;
pub use settings_database::{
    SETTINGS_DATABASE_FILE_NAME, read_push_to_talk_delay, settings_database_path,
};
