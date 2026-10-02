//! The 1.0 plugin's settings: the values `SETCFG` sets and the defaults.
//!
//! **Role:** the 1.0 plugin's settings model: every value the game sets with `SETCFG`, its default
//! and its type, the key matching and conversions of the original, and the refresh flag behind
//! `NEEDCFG`.
//!
//! **Position:** `crates/version_1_0`, tier 0, depending on no workspace crate; used by
//! `tfar_client_state_1_0`, `tfar_game_commands_1_0`, `tfar_plugin_1_0`, `tfar_radio_routing_1_0`,
//! `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** none of its own; `PluginSettings` is a value type the plugin owns and
//! guards.
//!
//! **Invariants:** an unknown key or a malformed value is an `Error` and leaves the settings
//! unchanged; every setting keeps the kind of its default.

mod crate_role;
mod error;
mod plugin_settings;
pub mod prelude;
mod setting_key;
mod setting_value;
mod type_tag;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
pub use plugin_settings::{PluginSettings, SettingChanged};
pub use setting_key::SettingKey;
pub use setting_value::{SettingKind, SettingValue};
pub use type_tag::TypeTag;
