//! The per-client state of the 1.0 plugin: positions, radios, tangents, vehicles and effect state.
//!
//! **Role:** the state the 1.0 plugin keeps per TeamSpeak client, keyed by real name: position,
//! radios, tangent state, vehicle and intercom, and the effect and HRTF state of each sender; the
//! game data of the local player; the game link's connection state; and the apply function of
//! each game and plugin message.
//!
//! **Position:** `crates/version_1_0`, tier 3, depending on `tfar_geometry`, `tfar_arma_values`,
//! `tfar_game_commands_1_0`, `tfar_plugin_commands_1_0`, `tfar_radio_effects_1_0`,
//! `tfar_plugin_settings_1_0`, `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`,
//! `tfar_sample_buffer`; used by `tfar_plugin_1_0`, `tfar_radio_routing_1_0`,
//! `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** a [`SessionState`] per server connection, kept by the plugin app behind
//! one lock; no lock and no thread of its own. Time comes in as `now`.
//!
//! **Invariants:** effect state is only reached while its session's lock is held; queries are
//! pure; DSP objects are created only by [`ClientEffects::ensure_dsp`] and the audio path borrows
//! them without allocating.

mod client_data;
mod client_directory;
mod crate_role;
mod data_frame;
mod effects;
mod error;
mod game_data;
mod game_link_state;
mod plugin_enabled;
pub mod prelude;
mod radio_volume;
mod receiving_transmissions;
mod reception;
mod session;
#[cfg(test)]
#[path = "tests/test_support.rs"]
mod test_support;

pub use client_data::{ALIVE_TIMEOUT, ClientData};
pub use client_directory::ClientDirectory;
pub use crate_role::CRATE_ROLE;
pub use data_frame::DataFrame;
pub use effects::{
    CANT_SPEAK_FILTER_KEY, CANT_SPEAK_LOW_PASS, ClientEffects, DspContext, DspRequirement,
    ObjectInterceptionFilter, PHONE_BAND_PASS, SPEAKER_BAND_PASS, VEHICLE_FILTER_KEY,
    VOICE_HRTF_KEY, object_interception_cutoff_hz, vehicle_low_pass,
};
pub use error::{Error, Result};
pub use game_data::{GameData, Speaker};
pub use game_link_state::{GameLinkPoll, GameLinkState, GameLinkTransition, TRANSITION_DEBOUNCE};
pub use plugin_enabled::PLUGIN_ENABLED_CACHE;
pub use radio_volume::{local_radio_sound, radio_volume_gain, remote_radio_sound};
pub use reception::{
    HeardVia, INTERCOM_RADIO_ID, IgnoreTangents, LOCAL_RADIO_ID, ReceivedOn, SendingRadio,
    TangentRadio, intercom_radio_id, local_radio_id,
};
pub use session::{
    KilledApplied, LocalReleaseAllApplied, LocalTangentApplied, LocalTangentSound, PositionApplied,
    RELEASED_FREQUENCY_PREFIX, ReleaseAllApplied, RemoteRadioSounds, RemoteSoundPlacement,
    RenameRequest, SessionState, TangentEffects, TangentTransition, VolumeApplied,
};
