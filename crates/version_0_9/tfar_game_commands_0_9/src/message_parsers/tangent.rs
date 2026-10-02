//! `TANGENT`, `TANGENT_LR` and `TANGENT_DD`: the local player pressed or released a radio's
//! push-to-talk.

use tfar_arma_values::RadioSubtype;

use crate::command_names::{TANGENT, TANGENT_DD, TANGENT_LR};
use crate::error::Result;
use crate::field_values::{exact_fields, rounded_integer};
use crate::game_command::GameCommand;

#[cfg(test)]
#[path = "tests/tangent.rs"]
mod tests;

/// The radio family a push-to-talk command keys, named by the command itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TangentKind {
    /// `TANGENT`: a short-range radio.
    ShortRange,
    /// `TANGENT_LR`: a long-range radio.
    LongRange,
    /// `TANGENT_DD`: a diver radio.
    DiverDigital,
}

impl TangentKind {
    /// The kind a command name keys, or `None` for a name that is not a push-to-talk command.
    pub fn from_command_name(command: &str) -> Option<Self> {
        match command {
            TANGENT => Some(Self::ShortRange),
            TANGENT_LR => Some(Self::LongRange),
            TANGENT_DD => Some(Self::DiverDigital),
            _ => None,
        }
    }

    /// The command name: `TANGENT`, `TANGENT_LR` or `TANGENT_DD`.
    pub const fn command_name(self) -> &'static str {
        match self {
            Self::ShortRange => TANGENT,
            Self::LongRange => TANGENT_LR,
            Self::DiverDigital => TANGENT_DD,
        }
    }
}

/// `<TANGENT|TANGENT_LR|TANGENT_DD>\t<PRESSED|RELEASED>\t<frequency>\t<range>\t<subtype>`:
/// 5 fields, or 4 for an empty subtype: a radio without `tf_subtype` sends an empty last field,
/// which `std::getline` drops, and it reads as [`RadioSubtype::Unknown`] of the empty text (no
/// radio sound). Anything but `PRESSED` is a release, and the range is rounded as the receiving
/// plugin rounds it (`parseArmaNumberToInt`).
pub(crate) fn parse_tangent(kind: TangentKind, fields: &[&str]) -> Result<GameCommand> {
    let command = kind.command_name();
    let [_, state, frequency, range, subtype] = match fields {
        [name, state, frequency, range] => [*name, *state, *frequency, *range, ""],
        _ => exact_fields::<5>(command, fields)?,
    };
    Ok(GameCommand::Tangent {
        kind,
        pressed: state == "PRESSED",
        frequency: frequency.to_owned(),
        range: rounded_integer(command, "range", range)?,
        subtype: RadioSubtype::parse(subtype),
    })
}
