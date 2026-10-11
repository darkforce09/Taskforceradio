//! The names a caller of `tfar_plugin_logic_0_9::client_state` imports with `use crate::client_state::prelude::*;`,
//! with the command value types the state carries.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::client_state::client_data::ClientData;
pub use crate::client_state::client_id::ClientId;
pub use crate::client_state::command_application::{
    PositionOutcome, RemoteTangent, RemoteTangentOutcome, RemoteVolumeOutcome, RenameRequest,
    SpeakingState, TangentSound,
};
pub use crate::client_state::dsp_state::{
    ClientDsp, DirectVoiceDsp, DirectVoiceRequirement, DspRequirements, RadioDsp, RadioEffectKind,
    RadioRequirement, SpeakerRequirement, VehicleFilter,
};
pub use crate::client_state::game_session::{
    GameEnd, GameSession, GameStart, GameTimeout, SeriousChannel,
};
pub use crate::client_state::server_radio_data::{
    FrequencySettings, INVALID_DATA_FRAME, ServerRadioData,
};
pub use crate::client_state::speaker_radio::SpeakerRadio;
pub use crate::client_state::vehicle_muffling::VehicleMuffling;
pub use crate::game_commands::prelude::{RadioSubtype, StereoMode};
pub use crate::game_commands::{TangentKind, UsableRadios, VehicleDescriptor};
pub use tfar_foundation::geometry::{Angle, Position};
