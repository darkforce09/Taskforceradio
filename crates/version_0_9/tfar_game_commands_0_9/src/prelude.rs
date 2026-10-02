//! The names a caller of `tfar_game_commands_0_9` imports with
//! `use tfar_game_commands_0_9::prelude::*;`, with the value types of `tfar_arma_values` and
//! `tfar_geometry` the commands carry.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`;
//! the reply strings and command names stay in [`crate::replies`] and [`crate::command_names`]
//! so their short names keep their module.

pub use crate::crate_role::CRATE_ROLE;
pub use crate::game_activity::{IN_GAME_TRIGGER_EXEMPT_PREFIX, message_counts_as_game_activity};
pub use crate::game_command::GameCommand;
pub use crate::message_parsers::{
    FrequencyUpdate, PositionUpdate, SpeakerRecord, TangentKind, TsInfoQuery, UsableRadios,
};
pub use crate::replies::reply_for_unhandled;
pub use crate::vehicle_descriptor::VehicleDescriptor;
pub use tfar_arma_values::{FrequencyEntry, RadioSubtype, StereoMode};
pub use tfar_geometry::{Angle, Position};
