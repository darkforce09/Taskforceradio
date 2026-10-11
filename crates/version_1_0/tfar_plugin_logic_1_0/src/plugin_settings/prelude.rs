//! The names a caller of `tfar_plugin_logic_1_0::plugin_settings` imports with `use crate::plugin_settings::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::plugin_settings::setting_key::SettingKey;
pub use crate::plugin_settings::setting_value::{SettingKind, SettingValue};
pub use crate::plugin_settings::settings_table::{PluginSettings, SettingChanged};
pub use crate::plugin_settings::type_tag::TypeTag;
