//! `PluginCommand`: the messages 0.9.12 plugins send each other through TeamSpeak.
//!
//! **Role:** encode and decode the three plugin commands that travel between the TeamSpeak
//! clients of one channel: push-to-talk changes, the release of every push-to-talk, and the
//! voice volume with the talking state.
//!
//! **Position:** a module of `tfar_plugin_commands_0_9`; the plugin app encodes before
//! `sendPluginCommand` (target `CURRENT_CHANNEL`) and decodes in `onPluginCommandEvent` after
//! `accepts_sender`.
//!
//! **Signals and state:** none; pure functions, called on whichever thread sends or receives.
//!
//! **Invariants:** the text is the original's byte for byte (`TANGENT` as the game command plus
//! `\t<nickname>`, plugin.cpp:1837; `RELEASE_ALL_TANGENTS` as forwarded, plugin.cpp:1787;
//! `VOLUME`, plugin.cpp:2906); decoding follows `processPluginCommand` (plugin.cpp:3071-3110):
//! fields split like `std::getline`, exact field counts, `PRESSED` and `true` compared exactly,
//! the range rounded and the voice volume truncated.

use tfar_arma_values::separators::FIELD;
use tfar_arma_values::{RadioSubtype, parse_int, parse_number, split_fields};

use crate::error::{Error, Result};

#[cfg(test)]
#[path = "tests/plugin_command.rs"]
mod tests;

/// The command name of a short-range push-to-talk change.
pub const TANGENT: &str = "TANGENT";
/// The command name of a long-range push-to-talk change.
pub const TANGENT_LR: &str = "TANGENT_LR";
/// The command name of a diver radio push-to-talk change.
pub const TANGENT_DD: &str = "TANGENT_DD";
/// The command name of the release of every push-to-talk of one player.
pub const RELEASE_ALL_TANGENTS: &str = "RELEASE_ALL_TANGENTS";
/// The command name of the voice volume and talking state of one player.
pub const VOLUME: &str = "VOLUME";

/// The push-to-talk state text of a pressed tangent.
const PRESSED: &str = "PRESSED";
/// The push-to-talk state text of a released tangent.
const RELEASED: &str = "RELEASED";

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

/// One 0.9.12 plugin command.
#[derive(Clone, Debug, PartialEq)]
pub enum PluginCommand {
    /// `<TANGENT|TANGENT_LR|TANGENT_DD>\t<PRESSED|RELEASED>\t<frequency>\t<range>\t<subtype>\t<nickname>`:
    /// a player pressed or released a radio's push-to-talk.
    Tangent {
        /// The radio family, from the command name.
        kind: TangentKind,
        /// Whether the push-to-talk is now pressed; on decode anything but `PRESSED` is a release.
        pressed: bool,
        /// The frequency with the radio code appended (`87.6_bluefor`), as the game wrote it.
        frequency: String,
        /// The radio's range in metres, rounded as `parseArmaNumberToInt` rounds it.
        range: i32,
        /// The kind of radio.
        subtype: RadioSubtype,
        /// The nickname of the player who keyed the radio.
        nickname: String,
    },
    /// `RELEASE_ALL_TANGENTS\t<nickname>`: every push-to-talk of the player ends.
    ReleaseAllTangents {
        /// The nickname of the player whose push-to-talk ends.
        nickname: String,
    },
    /// `VOLUME\t<nickname>\t<voice volume>\t<true|false>`: the distance a player's voice carries
    /// and whether they talk now.
    Volume {
        /// The nickname of the player.
        nickname: String,
        /// The distance in metres the player's voice carries.
        voice_volume: i32,
        /// Whether the player talks now (`true`).
        is_talking: bool,
    },
}

impl PluginCommand {
    /// The command as text, byte for byte as a Windows 0.9.12 plugin sends it.
    pub fn encode(&self) -> String {
        match self {
            Self::Tangent {
                kind,
                pressed,
                frequency,
                range,
                subtype,
                nickname,
            } => {
                let state = if *pressed { PRESSED } else { RELEASED };
                let name = kind.command_name();
                let subtype = subtype.as_str();
                format!("{name}\t{state}\t{frequency}\t{range}\t{subtype}\t{nickname}")
            }
            Self::ReleaseAllTangents { nickname } => format!("{RELEASE_ALL_TANGENTS}\t{nickname}"),
            Self::Volume {
                nickname,
                voice_volume,
                is_talking,
            } => format!("{VOLUME}\t{nickname}\t{voice_volume}\t{is_talking}"),
        }
    }

    /// The command in `text`, as `processPluginCommand` in
    /// `reference/tfar_0_9_12/ts/src/plugin.cpp` reads it: a tangent needs exactly 6 fields,
    /// `RELEASE_ALL_TANGENTS` 2 and `VOLUME` 4.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownCommand`] for any other name or count, and [`Error::Malformed`] for a range
    /// or voice volume that is not a number (where the original's `std::stoi` would throw).
    pub fn decode(text: &str) -> Result<Self> {
        let fields = split_fields(text, FIELD);
        let unknown = || Error::UnknownCommand {
            command: text.to_owned(),
            found: fields.len(),
        };
        match fields.as_slice() {
            [command, state, frequency, range, subtype, nickname] => {
                let kind = TangentKind::from_command_name(command).ok_or_else(unknown)?;
                Ok(Self::Tangent {
                    kind,
                    pressed: *state == PRESSED,
                    frequency: (*frequency).to_owned(),
                    range: parse_int(range).map_err(|source| Error::Malformed {
                        command: kind.command_name(),
                        field: "range",
                        source,
                    })?,
                    subtype: RadioSubtype::parse(subtype),
                    nickname: (*nickname).to_owned(),
                })
            }
            [command, nickname] if *command == RELEASE_ALL_TANGENTS => {
                Ok(Self::ReleaseAllTangents {
                    nickname: (*nickname).to_owned(),
                })
            }
            [command, nickname, voice_volume, is_talking] if *command == VOLUME => {
                Ok(Self::Volume {
                    nickname: (*nickname).to_owned(),
                    voice_volume: truncated_voice_volume(voice_volume)?,
                    is_talking: *is_talking == "true",
                })
            }
            _ => Err(unknown()),
        }
    }
}

/// The voice volume field of `VOLUME`, truncated toward zero as `std::stoi` truncates it.
fn truncated_voice_volume(text: &str) -> Result<i32> {
    let malformed = |source| Error::Malformed {
        command: VOLUME,
        field: "voice_volume",
        source,
    };
    let truncated = parse_number(text).map_err(malformed)?.trunc();
    if truncated < f64::from(i32::MIN) || truncated > f64::from(i32::MAX) {
        return Err(malformed(tfar_arma_values::Error::IntegerOutOfRange {
            value: text.to_owned(),
        }));
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the value is integral and checked to lie within the i32 range"
    )]
    Ok(truncated as i32)
}
