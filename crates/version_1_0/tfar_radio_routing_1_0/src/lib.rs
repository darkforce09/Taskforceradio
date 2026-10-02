//! Whether and how a 1.0 listener hears a speaker: direct, over radio, intercom or speaker radio.
//!
//! **Role:** the 1.0 reception rules: `isOverRadio` and its neighbours decide whether a listener
//! hears a speaker over radio (with antennas and underwater range), over intercom or through a
//! speaker radio, and with which error level; the serious-mode mute rules and the record of who
//! is muted; the serious channel lookup.
//!
//! **Position:** `crates/version_1_0`, tier 4, depending on `tfar_client_state_1_0`,
//! `tfar_antennas_1_0`, `tfar_plugin_settings_1_0`, `tfar_geometry`, `tfar_arma_values`; used by
//! `tfar_plugin_1_0`, `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** none but [`MuteTracker`], which the plugin app keeps beside its session
//! state; the rules read a [`RoutingContext`] built under the session lock.
//!
//! **Invariants:** routing reads client state and never changes it; the local player is never
//! muted and never hears itself over its own radio.

mod crate_role;
mod effect_error;
mod error;
#[cfg(test)]
#[path = "tests/fixtures.rs"]
mod fixtures;
mod mute;
pub mod prelude;
mod reception;
mod remote_tangent;
mod routing_context;
mod serious_mode;
mod underwater;

pub use crate_role::CRATE_ROLE;
pub use effect_error::{capped_by_antenna, effect_error_from_distance, underwater_effect_error};
pub use error::{Error, Result};
pub use mute::{
    MuteChanges, MuteDecision, MuteTracker, VOICE_RANGE_MARGIN, killed_mute_matrix, mute_decision,
};
pub use reception::{is_over_local_radio, is_over_radio};
pub use remote_tangent::apply_remote_tangent;
pub use routing_context::RoutingContext;
pub use serious_mode::{CHANNEL_ID_PREFIX, ChannelId, serious_mode_channel};
pub use underwater::{diver_radio_distance, underwater_weighted_distance};
