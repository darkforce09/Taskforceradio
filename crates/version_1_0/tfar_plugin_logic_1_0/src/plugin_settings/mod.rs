//! The 1.0 plugin's settings: the values `SETCFG` sets and the defaults.
//!
//! **Role:** the 1.0 plugin's settings model: every value the game sets with `SETCFG`, its default
//! and its type, the key matching and conversions of the original, and the refresh flag behind
//! `NEEDCFG`.
//!
//! **Position:** a module of `tfar_plugin_logic_1_0`, depending on no workspace crate; used by
//! `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_1_0::game_commands`, `tfar_plugin_1_0`, `tfar_plugin_logic_1_0::radio_routing`,
//! `tfar_plugin_logic_1_0::voice_pipeline`.
//!
//! **Signals and state:** none of its own; `PluginSettings` is a value type the plugin owns and
//! guards.
//!
//! **Invariants:** an unknown key or a malformed value is an `Error` and leaves the settings
//! unchanged; every setting keeps the kind of its default.

mod error;
pub mod prelude;
mod setting_key;
mod setting_value;
mod settings_table;
mod type_tag;

pub use error::{Error, Result};
pub use setting_key::SettingKey;
pub use setting_value::{SettingKind, SettingValue};
pub use settings_table::{PluginSettings, SettingChanged};
pub use type_tag::TypeTag;
