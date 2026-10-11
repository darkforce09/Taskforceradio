//! The per-client state of the 0.9.12 plugin: positions, radios, tangents and effect state.
//!
//! **Role:** the state the 0.9.12 plugin keeps per server connection and per TeamSpeak client,
//! keyed by nickname: positions, radios, push-to-talk, voice volume, the game session and the
//! sound state of each sender, with the application of every game message and plugin command.
//!
//! **Position:** `crates/version_0_9`, tier 3, depending on `tfar_geometry`,
//! `tfar_game_commands_0_9`, `tfar_radio_effects_0_9`, `tfar_dsp_filters`, `tfar_dynamics`,
//! `tfar_hrtf`; used by `tfar_plugin_0_9`, `tfar_radio_routing_0_9`, `tfar_voice_pipeline_0_9`.
//!
//! **Signals and state:** pure state and decisions: no TeamSpeak call, no thread, no clock
//! (time comes in as `now`). The plugin owns one [`ServerRadioData`] per server connection
//! behind one lock; sound state is made on the command threads ([`ClientData::ensure_dsp`]) and
//! only borrowed on the audio thread.
//!
//! **Invariants:** every field starts at a defined value; a lookup by client id returns only a
//! current entry; the original nickname is saved once per game session; a game starts on the
//! first game activity whatever the serious channel.

mod client_data;
mod client_id;
mod command_application;
mod data_frames;
mod dsp_state;
mod error;
mod game_session;
mod nickname_list;
pub mod prelude;
mod server_radio_data;
mod speaker_radio;
mod vehicle_muffling;

pub use client_data::{ClientData, PLUGIN_ENABLED_GRACE};
pub use client_id::ClientId;
pub use command_application::{
    NICKNAMES_NEVER_ADOPTED, PositionOutcome, RemoteTangent, RemoteTangentOutcome,
    RemoteVolumeOutcome, RenameRequest, SpeakingState, TangentSound,
};
pub use data_frames::{DATA_FRAME_WINDOW, POSITION_EXPIRY};
pub use dsp_state::{
    CANT_SPEAK_LOW_PASS, ClientDsp, DirectVoiceDsp, DirectVoiceRequirement, DspRequirements,
    FILTER_SAMPLE_RATE_HZ, PHONE_BAND_PASS, RadioDsp, RadioEffectKind, RadioRequirement,
    SPEAKER_BAND_PASS, SpeakerRequirement, VehicleFilter, vehicle_low_pass,
};
pub use error::{Error, Result};
pub use game_session::{
    GAME_ACTIVITY_TIMEOUT, GameEnd, GameSession, GameStart, GameTimeout, SeriousChannel,
};
pub use server_radio_data::{FrequencySettings, INVALID_DATA_FRAME, ServerRadioData};
pub use speaker_radio::{SpeakerRadio, UNDER_WATER_WAVE_Z, is_under_water};
pub use vehicle_muffling::{MAX_COMBINED_ISOLATION, MUFFLING_THRESHOLD, VehicleMuffling};
