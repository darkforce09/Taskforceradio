//! The names a caller of `tfar_game_commands_1_0` imports with `use tfar_game_commands_1_0::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`;
//! the reply strings stay in [`crate::replies`] so their short names keep their module.

pub use crate::command_name::CommandName;
pub use crate::commands::{
    ConfigAssignment, FrequencyReport, NICKNAMES_NEVER_ADOPTED, PositionUpdate, SpeakerRecord,
    TANGENT_PRESSED, TANGENT_RELEASED, TangentKind, TangentPress, TowerRecord,
    TrackerCustomVariable, TrackingEvent, TsInfoQuery,
};
pub use crate::fields::{NetworkObjectId, VehicleDescriptor};
pub use crate::game_command::GameCommand;
pub use crate::parse::parse;
