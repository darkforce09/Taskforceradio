//! The 0.9.12 messages Arma sends the plugin, parsed into typed commands.
//!
//! **Role:** the typed form of every 0.9.12 game message (`POS`, `TANGENT`, `FREQ`, `KILLED`,
//! `TRACK` and the rest), parsed with checked field counts, and the reply strings the plugin
//! answers with.
//!
//! **Position:** `crates/version_0_9`, tier 2, depending on `tfar_arma_values`, `tfar_geometry`;
//! used by `tfar_client_state_0_9`, `tfar_plugin_0_9`.
//!
//! **Signals and state:** none; pure parsing, callable from any thread.
//!
//! **Invariants:** every field access is length-checked and every number parsed through
//! `tfar_arma_values`, so a short or malformed message is an `Error`, never a panic.

pub mod command_names;
mod error;
mod field_values;
mod game_activity;
mod game_command;
mod message_parsers;
pub mod prelude;
pub mod replies;
mod vehicle_descriptor;

pub use error::{Error, Result};
pub use game_activity::{IN_GAME_TRIGGER_EXEMPT_PREFIX, message_counts_as_game_activity};
pub use game_command::GameCommand;
pub use message_parsers::{
    FrequencyUpdate, PositionUpdate, SpeakerRecord, TangentKind, TsInfoQuery, UsableRadios,
};
pub use replies::reply_for_unhandled;
pub use vehicle_descriptor::VehicleDescriptor;
