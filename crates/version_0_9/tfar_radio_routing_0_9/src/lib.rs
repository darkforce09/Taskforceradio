//! Whether and how a 0.9.12 listener hears a speaker: direct, over radio or over a speaker radio.
//!
//! **Role:** the 0.9.12 reception rules (`isOverLocalRadio`, `isOverRadio` and the distances
//! they compare), the mute decisions with the tracker of what the plugin muted, the serious-mode
//! channel checks, and the sound state a sender's voice needs.
//!
//! **Position:** `crates/version_0_9`, tier 4, depending on `tfar_client_state_0_9`,
//! `tfar_geometry`; used by `tfar_plugin_0_9`, `tfar_voice_pipeline_0_9`.
//!
//! **Signals and state:** pure decisions over a borrowed `ServerRadioData`: no TeamSpeak call,
//! no thread, no clock. The two `set_*` functions also change what their decision names (a
//! muted player's voice state, the `MuteTracker`).
//!
//! **Invariants:** the check order and overwrite rule of the original; an unmute goes only to a
//! client this plugin muted, and a request only on a change; a missing entry is muted without
//! touching voice state.

mod crate_role;
mod distances;
mod dsp_planning;
mod error;
mod heard_via;
mod mute_decisions;
pub mod prelude;
mod reception;
mod serious_mode;

pub use crate_role::CRATE_ROLE;
pub use distances::{
    DIVER_RADIO_MAX_DISTANCE, DIVER_RADIO_MIN_DISTANCE, diver_radio_distance,
    effect_error_from_distance, effective_distance, raw_distance,
};
pub use dsp_planning::dsp_requirements;
pub use error::{Error, Result};
pub use heard_via::{HeardOn, HeardVia, IgnoreTangents, LOCAL_RADIO_ID};
pub use mute_decisions::{
    GameClientMute, MuteDecision, MuteQuery, MuteTracker, game_client_mute,
    set_game_client_mute_status, set_mute_for_dead_players,
};
pub use reception::{is_over_local_radio, is_over_radio, receptions};
pub use serious_mode::{
    OTHER_RADIO_PLUGIN_CHANNELS, is_other_radio_plugin_channel, is_serious_mode,
};
