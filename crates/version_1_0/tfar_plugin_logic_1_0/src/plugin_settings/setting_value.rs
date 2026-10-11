//! `SettingValue` and `SettingKind`: the value of one setting and the conversions `SETCFG` applies.
//!
//! **Role:** hold a boolean, number or text value and convert an incoming value into the kind of
//! the setting it is set on, as `settingValue::operator=` in
//! `reference/tfar_1_0/ts/src/settings.hpp:80-101` converts.
//!
//! **Position:** a module of `tfar_plugin_logic_1_0::plugin_settings`; used by `SettingKey` for the
//! defaults, by `TypeTag` for the incoming value and by `PluginSettings::set`.
//!
//! **Signals and state:** none; a value type and pure functions, callable from any thread.
//!
//! **Invariants:** a boolean set from a number is `number > 0`; a number set from a boolean is 1 or
//! 0; a text set from a boolean is `true` or `false` and from a number the `%f` text with six
//! decimals; a text value never converts into a boolean or a number.

#[cfg(test)]
#[path = "tests/setting_value.rs"]
mod tests;

/// The kind of value a setting holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettingKind {
    /// A boolean.
    Bool,
    /// A 32-bit float, as the original stores it.
    Number,
    /// A text.
    Text,
}

/// The value of one setting, or the value a `SETCFG` message carries before it is converted.
#[derive(Clone, Debug, PartialEq)]
pub enum SettingValue {
    /// A boolean value.
    Bool(bool),
    /// A number, a 32-bit float as the original stores it.
    Number(f32),
    /// A text value.
    Text(String),
}

impl SettingValue {
    /// The kind of this value.
    pub const fn kind(&self) -> SettingKind {
        match self {
            Self::Bool(_) => SettingKind::Bool,
            Self::Number(_) => SettingKind::Number,
            Self::Text(_) => SettingKind::Text,
        }
    }

    /// This value as a boolean setting takes it: a boolean as it is, a number as `number > 0`;
    /// `None` for a text, which the original cannot store in a boolean setting.
    pub fn into_bool(self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(value),
            Self::Number(value) => Some(value > 0.0),
            Self::Text(_) => None,
        }
    }

    /// This value as a number setting takes it: a number as it is, a boolean as 1 or 0; `None`
    /// for a text, which the original cannot store in a number setting.
    pub fn into_number(self) -> Option<f32> {
        match self {
            Self::Bool(value) => Some(if value { 1.0 } else { 0.0 }),
            Self::Number(value) => Some(value),
            Self::Text(_) => None,
        }
    }

    /// This value as a text setting takes it, the text `operator std::string` gives: `true` or
    /// `false` for a boolean, `std::to_string(float)` (`%f`, six decimals) for a number.
    pub fn into_text(self) -> String {
        match self {
            Self::Bool(value) => boolean_text(value).to_owned(),
            Self::Number(value) => number_text(value),
            Self::Text(value) => value,
        }
    }

    /// This value converted into `kind`, or `None` where the original has no conversion (a text
    /// into a boolean or a number).
    pub fn converted_to(self, kind: SettingKind) -> Option<Self> {
        match kind {
            SettingKind::Bool => self.into_bool().map(Self::Bool),
            SettingKind::Number => self.into_number().map(Self::Number),
            SettingKind::Text => Some(Self::Text(self.into_text())),
        }
    }
}

/// `true` or `false`, as the original writes a boolean as text.
const fn boolean_text(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

/// `value` as `std::to_string(float)` writes it: the `%f` format of the value widened to a
/// double, six decimals.
fn number_text(value: f32) -> String {
    format!("{:.6}", f64::from(value))
}
