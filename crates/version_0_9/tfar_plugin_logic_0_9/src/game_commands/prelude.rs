//! The names a caller of `tfar_plugin_logic_0_9::game_commands` imports with
//! `use crate::game_commands::prelude::*;`, with the value types of `tfar_foundation::arma_values` and
//! `tfar_foundation::geometry` the commands carry.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`;
//! the reply strings and command names stay in [`crate::game_commands::replies`] and [`crate::game_commands::command_names`]
//! so their short names keep their module.

pub use crate::game_commands::game_activity::{
    IN_GAME_TRIGGER_EXEMPT_PREFIX, message_counts_as_game_activity,
};
pub use crate::game_commands::game_command::GameCommand;
pub use crate::game_commands::message_parsers::{
    FrequencyUpdate, PositionUpdate, SpeakerRecord, TangentKind, TsInfoQuery, UsableRadios,
};
pub use crate::game_commands::replies::reply_for_unhandled;
pub use crate::game_commands::vehicle_descriptor::VehicleDescriptor;
pub use tfar_foundation::arma_values::{FrequencyEntry, RadioSubtype, StereoMode};
pub use tfar_foundation::geometry::{Angle, Position};
