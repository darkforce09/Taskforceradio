//! The names a caller of `tfar_plugin_logic_1_0::game_commands` imports with `use crate::game_commands::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`;
//! the reply strings stay in [`crate::game_commands::replies`] so their short names keep their module.

pub use crate::game_commands::command_name::CommandName;
pub use crate::game_commands::commands::{
    ConfigAssignment, FrequencyReport, NICKNAMES_NEVER_ADOPTED, PositionUpdate, SpeakerRecord,
    TANGENT_PRESSED, TANGENT_RELEASED, TangentKind, TangentPress, TowerRecord,
    TrackerCustomVariable, TrackingEvent, TsInfoQuery,
};
pub use crate::game_commands::fields::{NetworkObjectId, VehicleDescriptor};
pub use crate::game_commands::game_command::GameCommand;
pub use crate::game_commands::parse::parse;
