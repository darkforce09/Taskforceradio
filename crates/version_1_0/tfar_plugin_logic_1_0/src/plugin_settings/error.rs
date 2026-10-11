//! The error type of `tfar_plugin_logic_1_0::plugin_settings`.

use crate::plugin_settings::setting_key::SettingKey;

/// Every failure `tfar_plugin_logic_1_0::plugin_settings` reports. A failed set leaves the settings unchanged.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    /// The `SETCFG` key names no setting. The message is the original's text, which the plugin
    /// shows the player (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:395-397`).
    #[error(
        "Used invalid config key: {key} (You probably need to update your TFAR Teamspeak plugin)"
    )]
    UnknownKey {
        /// The key text the game sent.
        key: String,
    },
    /// A `SCALAR` value is not a finite number.
    #[error("`{value}` is not a finite number")]
    MalformedNumber {
        /// The value text the game sent.
        value: String,
    },
    /// A text value was set on a boolean or number setting, which the original cannot store.
    #[error("the text `{value}` cannot be stored in the {} setting `{}`", kind_name(*key), key.wire_name())]
    TextForTypedSetting {
        /// The setting the value was set on.
        key: SettingKey,
        /// The text that was set.
        value: String,
    },
}

/// The kind word of `key` for an error message.
fn kind_name(key: SettingKey) -> &'static str {
    match key.kind() {
        crate::plugin_settings::setting_value::SettingKind::Bool => "boolean",
        crate::plugin_settings::setting_value::SettingKind::Number => "number",
        crate::plugin_settings::setting_value::SettingKind::Text => "text",
    }
}

/// The result of a fallible `tfar_plugin_logic_1_0::plugin_settings` operation.
pub type Result<T> = core::result::Result<T, Error>;
