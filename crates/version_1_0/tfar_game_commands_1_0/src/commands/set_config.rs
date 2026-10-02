//! `ConfigAssignment`: the `SETCFG` message, one plugin setting.

use tfar_plugin_settings_1_0::TypeTag;

use crate::error::Result;
use crate::fields::MessageFields;

#[cfg(test)]
#[path = "tests/set_config.rs"]
mod tests;

/// One `SETCFG` message (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:388-414`), applied with
/// `tfar_plugin_settings_1_0::PluginSettings::set`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigAssignment {
    /// The setting's key as sent; the settings match it by case-insensitive prefix.
    pub key: String,
    /// The value text as sent.
    pub value: String,
    /// The type tag of a 4-token message; `None` for any other count, where the original sets
    /// the empty text instead of the value.
    pub type_tag: Option<TypeTag>,
}

/// The `SETCFG` message whose tokens are `fields`: at least the key and the value.
pub(crate) fn parse_config_assignment(fields: &MessageFields<'_>) -> Result<ConfigAssignment> {
    fields.require_at_least(3)?;
    let type_tag = if fields.count() == 4 {
        Some(TypeTag::from_wire(fields.text(3)?))
    } else {
        None
    };
    Ok(ConfigAssignment {
        key: fields.text(1)?.to_owned(),
        value: fields.text(2)?.to_owned(),
        type_tag,
    })
}
