//! The names a caller of `tfar_client_state_1_0` imports with `use tfar_client_state_1_0::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.
//! The prelude also re-exports the types of other workspace crates that appear in this crate's
//! public fields, so a user names them without depending on those crates.

pub use crate::client_data::{ALIVE_TIMEOUT, ClientData};
pub use crate::client_directory::ClientDirectory;
pub use crate::crate_role::CRATE_ROLE;
pub use crate::data_frame::DataFrame;
pub use crate::effects::{
    CANT_SPEAK_FILTER_KEY, ClientEffects, DspContext, DspRequirement, ObjectInterceptionFilter,
    VEHICLE_FILTER_KEY, VOICE_HRTF_KEY, object_interception_cutoff_hz,
};
pub use crate::game_data::{GameData, Speaker};
pub use crate::game_link_state::{
    GameLinkPoll, GameLinkState, GameLinkTransition, TRANSITION_DEBOUNCE,
};
pub use crate::plugin_enabled::PLUGIN_ENABLED_CACHE;
pub use crate::radio_volume::{radio_volume_gain, remote_radio_sound};
pub use crate::reception::{
    HeardVia, INTERCOM_RADIO_ID, IgnoreTangents, LOCAL_RADIO_ID, ReceivedOn, SendingRadio,
    TangentRadio,
};
pub use crate::session::{
    KilledApplied, PositionApplied, ReleaseAllApplied, RemoteRadioSounds, RemoteSoundPlacement,
    RenameRequest, SessionState, TangentEffects, TangentTransition, VolumeApplied,
};
pub use tfar_arma_values::{FrequencyEntry, RadioSubtype, StereoMode};
pub use tfar_game_commands_1_0::VehicleDescriptor;
pub use tfar_plugin_commands_1_0::{TangentBroadcast, TangentKind, TeamspeakClientId};
