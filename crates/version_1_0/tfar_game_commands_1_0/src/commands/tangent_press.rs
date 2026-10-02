//! `TangentPress`: the `TANGENT`, `TANGENT_LR` and `TANGENT_DD` messages, a radio press or release.
//!
//! **Role:** read a push-to-talk message into its kind, state, frequency, range, subtype and
//! optional radio class name.
//!
//! **Position:** a module of the crate's `commands`; built on `MessageFields` and
//! `tfar_arma_values::RadioSubtype`; called by the parser.
//!
//! **Signals and state:** none; pure parsing.
//!
//! **Invariants:** the fields are those of `CommandProcessor.cpp:293-376`
//! (`reference/tfar_1_0/ts/src/`): the message is 5 tokens, or 6 with the radio class name; only
//! `PRESSED` is a press; the subtype is matched by exact name; the range text is kept as sent so
//! the broadcast to other players carries the same bytes.

use tfar_arma_values::RadioSubtype;

use crate::command_name::CommandName;
use crate::error::Result;
use crate::fields::MessageFields;

#[cfg(test)]
#[path = "tests/tangent_press.rs"]
mod tests;

/// The kind of radio a tangent message keys, from its command name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TangentKind {
    /// `TANGENT`: a short-range radio.
    ShortRange,
    /// `TANGENT_LR`: a long-range radio.
    LongRange,
    /// `TANGENT_DD`: a diver radio, which the plugin handles as a short-range one.
    DiverDigital,
}

impl TangentKind {
    /// The command name of this kind: `TANGENT`, `TANGENT_LR` or `TANGENT_DD`.
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::ShortRange => "TANGENT",
            Self::LongRange => "TANGENT_LR",
            Self::DiverDigital => "TANGENT_DD",
        }
    }

    /// Whether the plugin transmits on its long-range radios for this kind; `TANGENT_DD` is not,
    /// it transmits as a short-range radio (`CommandProcessor.cpp:316`).
    pub const fn is_long_range(self) -> bool {
        matches!(self, Self::LongRange)
    }

    /// The command name of this kind.
    pub const fn command_name(self) -> CommandName {
        match self {
            Self::ShortRange => CommandName::Tangent,
            Self::LongRange => CommandName::TangentLongRange,
            Self::DiverDigital => CommandName::TangentDiverDigital,
        }
    }
}

/// The text of a press in the second field; any other text is a release.
pub const TANGENT_PRESSED: &str = "PRESSED";

/// The text of a release in the second field.
pub const TANGENT_RELEASED: &str = "RELEASED";

/// One push-to-talk message from the game.
#[derive(Clone, Debug, PartialEq)]
pub struct TangentPress {
    /// The kind of radio keyed.
    pub kind: TangentKind,
    /// The radio is pressed (the field is exactly `PRESSED`); released otherwise.
    pub is_pressed: bool,
    /// The frequency with the radio code appended, as the game writes it.
    pub frequency: String,
    /// The transmission range in metres, rounded.
    pub range: i32,
    /// The range as the game sent it, forwarded unchanged in the broadcast.
    pub range_text: String,
    /// The radio subtype, matched by exact name.
    pub subtype: RadioSubtype,
    /// The radio's class name, sent with a press only; the plugin uses it in half-duplex mode.
    pub radio_classname: Option<String>,
}

/// The tangent message of `kind` whose tokens are `fields`.
pub(crate) fn parse_tangent_press(
    kind: TangentKind,
    fields: &MessageFields<'_>,
) -> Result<TangentPress> {
    fields.require_between(5, 6)?;
    let range_text = fields.text(3)?;
    Ok(TangentPress {
        kind,
        is_pressed: fields.text(1)? == TANGENT_PRESSED,
        frequency: fields.text(2)?.to_owned(),
        range: fields.integer(3, "range")?,
        range_text: range_text.to_owned(),
        subtype: RadioSubtype::parse(fields.text(4)?),
        radio_classname: fields.text(5).ok().map(str::to_owned),
    })
}
