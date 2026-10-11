//! The 1.0 messages Arma sends the plugin, parsed into typed commands.
//!
//! **Role:** the typed form of every 1.0 game message (`POS`, `TANGENT`, `FREQ`, `SETCFG`,
//! `MISSIONEND`, data frames and the rest), parsed with checked token counts, and the answers
//! the plugin gives sync calls.
//!
//! **Position:** a module of `tfar_plugin_logic_1_0`, depending on `tfar_foundation::arma_values`, `tfar_foundation::geometry`,
//! `tfar_plugin_logic_1_0::plugin_settings`; used by `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_1_0`.
//!
//! **Signals and state:** none; pure parsing, callable from any thread.
//!
//! **Invariants:** a short message or a malformed field is an `Error`, never a panic or a silent
//! zero; `TANGENT` parses by its own token layout, never by shifted indices; every reply string
//! is the original's, byte for byte.

mod command_name;
mod commands;
mod error;
mod fields;
mod game_command;
mod parse;
pub mod prelude;
pub mod replies;

pub use command_name::CommandName;
pub use commands::{
    ConfigAssignment, FrequencyReport, NICKNAMES_NEVER_ADOPTED, PositionUpdate, SpeakerRecord,
    TANGENT_PRESSED, TANGENT_RELEASED, TangentKind, TangentPress, TowerRecord,
    TrackerCustomVariable, TrackingEvent, TsInfoQuery,
};
pub use error::{Error, Result};
pub use fields::{NetworkObjectId, VehicleDescriptor};
pub use game_command::GameCommand;
pub use parse::parse;
