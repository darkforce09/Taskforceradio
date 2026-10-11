//! Whether and how a 1.0 listener hears a speaker: direct, over radio, intercom or speaker radio.
//!
//! **Role:** the 1.0 reception rules: `isOverRadio` and its neighbours decide whether a listener
//! hears a speaker over radio (with antennas and underwater range), over intercom or through a
//! speaker radio, and with which error level; the serious-mode mute rules and the record of who
//! is muted; the serious channel lookup.
//!
//! **Position:** a module of `tfar_plugin_logic_1_0`, depending on `tfar_plugin_logic_1_0::client_state`,
//! `tfar_plugin_logic_1_0::antennas`, `tfar_plugin_logic_1_0::plugin_settings`, `tfar_foundation::geometry`, `tfar_foundation::arma_values`; used by
//! `tfar_plugin_1_0`, `tfar_plugin_logic_1_0::voice_pipeline`.
//!
//! **Signals and state:** none but [`MuteTracker`], which the plugin app keeps beside its session
//! state; the rules read a [`RoutingContext`] built under the session lock.
//!
//! **Invariants:** routing reads client state and never changes it; the local player is never
//! muted and never hears itself over its own radio.

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

pub use effect_error::{capped_by_antenna, effect_error_from_distance, underwater_effect_error};
pub use error::{Error, Result};
pub use mute::{
    MuteChanges, MuteDecision, MuteTracker, VOICE_RANGE_MARGIN, killed_mute_matrix, mute_decision,
};
pub use reception::{is_over_local_radio, is_over_radio, is_over_radio_into};
pub use remote_tangent::apply_remote_tangent;
pub use routing_context::RoutingContext;
pub use serious_mode::{CHANNEL_ID_PREFIX, ChannelId, serious_mode_channel};
pub use underwater::{diver_radio_distance, underwater_weighted_distance};
