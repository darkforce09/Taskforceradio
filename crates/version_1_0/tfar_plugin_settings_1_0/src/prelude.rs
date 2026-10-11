//! The names a caller of `tfar_plugin_settings_1_0` imports with `use tfar_plugin_settings_1_0::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::plugin_settings::{PluginSettings, SettingChanged};
pub use crate::setting_key::SettingKey;
pub use crate::setting_value::{SettingKind, SettingValue};
pub use crate::type_tag::TypeTag;
