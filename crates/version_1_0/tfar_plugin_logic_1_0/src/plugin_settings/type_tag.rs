//! `TypeTag`: the type name the addon writes after a `SETCFG` value, and how it reads the value.

use crate::plugin_settings::error::{Error, Result};
use crate::plugin_settings::setting_value::SettingValue;

#[cfg(test)]
#[path = "tests/type_tag.rs"]
mod tests;

/// The fourth field of a `SETCFG` message: the SQF `typeName` of the value
/// (`reference/tfar_1_0/addons/core/functions/plugin/fnc_setPluginSetting.sqf`), read as
/// `reference/tfar_1_0/ts/src/CommandProcessor.cpp:402-410` reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeTag {
    /// `BOOL`: the value is `true` only when it is exactly `true` or `TRUE`.
    Bool,
    /// `SCALAR`: the value is a number.
    Scalar,
    /// Any other tag (`STRING` among them): the value is text.
    Text,
}

impl TypeTag {
    /// The tag `tag` names, matched exactly: `BOOL`, `SCALAR`, and any other text as
    /// [`TypeTag::Text`].
    pub fn from_wire(tag: &str) -> Self {
        match tag {
            "BOOL" => Self::Bool,
            "SCALAR" => Self::Scalar,
            _ => Self::Text,
        }
    }

    /// The value `value_text` carries under this tag.
    ///
    /// # Errors
    ///
    /// [`Error::MalformedNumber`] for a `SCALAR` value that is not one finite decimal number
    /// (leading and trailing spaces allowed).
    pub fn read_value(self, value_text: &str) -> Result<SettingValue> {
        match self {
            Self::Bool => Ok(SettingValue::Bool(
                value_text == "true" || value_text == "TRUE",
            )),
            Self::Scalar => parse_scalar(value_text).map(SettingValue::Number),
            Self::Text => Ok(SettingValue::Text(value_text.to_owned())),
        }
    }
}

/// The number in `text`, as Arma's `str()` writes it, read as the 32-bit float the original
/// stores.
fn parse_scalar(text: &str) -> Result<f32> {
    let malformed = || Error::MalformedNumber {
        value: text.to_owned(),
    };
    let number: f32 = text.trim_matches(' ').parse().map_err(|_| malformed())?;
    if number.is_finite() {
        Ok(number)
    } else {
        Err(malformed())
    }
}
