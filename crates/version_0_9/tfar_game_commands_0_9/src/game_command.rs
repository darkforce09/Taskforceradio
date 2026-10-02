//! `GameCommand`: every message the 0.9.12 game sends the plugin, typed.
//!
//! **Role:** split a message into its tab-separated fields and hand it to the parser its command
//! name selects.
//!
//! **Position:** the entry point of `tfar_game_commands_0_9`; the plugin app parses every game
//! link message here, then performs the side effect and answers with a string from `replies`.
//!
//! **Signals and state:** none; pure parsing, called on the game link thread.
//!
//! **Invariants:** fields are split as `std::getline` splits them (a trailing empty field is
//! dropped); a known command with a field count its rule refuses is an error, never a guess;
//! a message whose command name is unknown is `GameCommand::Unknown` holding the whole message.

use tfar_arma_values::separators::FIELD;
use tfar_arma_values::{RadioSubtype, split_fields};

use crate::command_names::{
    FREQ, IS_SPEAKING, KILLED, POS, RELEASE_ALL_TANGENTS, SPEAKERS, TRACK, TS_INFO, VERSION,
};
use crate::error::Result;
use crate::game_activity::message_counts_as_game_activity;
use crate::message_parsers::{
    FrequencyUpdate, PositionUpdate, SpeakerRecord, TangentKind, TsInfoQuery, parse_frequencies,
    parse_is_speaking, parse_killed, parse_release_all_tangents, parse_speakers, parse_tangent,
    parse_track, parse_ts_info, parse_unit_position, parse_version,
};

#[cfg(test)]
#[path = "tests/game_command.rs"]
mod tests;

/// One message from the 0.9.12 game, as `processGameCommand` in
/// `reference/tfar_0_9_12/ts/src/plugin.cpp` reads it.
#[derive(Clone, Debug, PartialEq)]
pub enum GameCommand {
    /// `TS_INFO`: a question about TeamSpeak.
    TsInfo(TsInfoQuery),
    /// `KILLED`: the named player died.
    Killed {
        /// The nickname of the player who died.
        nickname: String,
    },
    /// `SPEAKERS`: the full list of loudspeaker radios near the local player; it replaces the
    /// previous list, and an empty list clears it.
    Speakers(Vec<SpeakerRecord>),
    /// `RELEASE_ALL_TANGENTS`: the named player's radios were taken, so every push-to-talk of
    /// theirs ends; the plugin forwards it to the other plugins in the channel.
    ReleaseAllTangents {
        /// The nickname of the player whose push-to-talk ends.
        nickname: String,
    },
    /// `VERSION`: the mission handshake.
    Version {
        /// The version of the TFAR addon the game runs.
        addon_version: String,
        /// The channel serious mode moves players into; empty for none.
        serious_channel_name: String,
        /// The password of the serious mode channel.
        serious_channel_password: String,
    },
    /// `TRACK`: an analytics request path for the plugin to fetch.
    Track {
        /// The request path and query, as sent.
        path: String,
    },
    /// `POS`: one player's position and abilities.
    Position(PositionUpdate),
    /// `TANGENT`, `TANGENT_LR` or `TANGENT_DD`: the local player pressed or released a radio's
    /// push-to-talk.
    Tangent {
        /// The radio family, from the command name.
        kind: TangentKind,
        /// Whether the push-to-talk is now pressed (`PRESSED`); anything else is a release.
        pressed: bool,
        /// The frequency with the radio code appended (`87.6_bluefor`), as sent.
        frequency: String,
        /// The radio's range in metres, rounded.
        range: i32,
        /// The kind of radio.
        subtype: RadioSubtype,
    },
    /// `FREQ`: the local player's radios and settings.
    Frequencies(FrequencyUpdate),
    /// `IS_SPEAKING`: whether the named player is talking.
    IsSpeaking {
        /// The nickname of the player asked about.
        nickname: String,
    },
    /// A message whose command name the plugin does not know, kept whole; answered `FAIL`.
    Unknown(String),
}

impl GameCommand {
    /// The command in `message`, split at tabs.
    ///
    /// | Command | Fields, name included |
    /// |---|---|
    /// | `TS_INFO`, `RELEASE_ALL_TANGENTS`, `TRACK`, `IS_SPEAKING` | exactly 2 |
    /// | `KILLED` | 2 or more |
    /// | `SPEAKERS` | 1 or more; records read from exactly 2 |
    /// | `VERSION` | 4, or 3 with an empty password |
    /// | `TANGENT`, `TANGENT_LR`, `TANGENT_DD` | 5, or 4 with an empty subtype |
    /// | `POS`, `FREQ` | exactly 14 |
    ///
    /// # Errors
    ///
    /// [`Error::WrongFieldCount`](crate::Error::WrongFieldCount) for a known command with a field
    /// count outside its rule, [`Error::Malformed`](crate::Error::Malformed) for a field that
    /// does not parse, and [`Error::ShortSpeakerRecord`](crate::Error::ShortSpeakerRecord) for a
    /// speaker record of fewer than 6 parts. Every error is answered
    /// [`FAIL`](crate::replies::FAIL).
    pub fn parse(message: &str) -> Result<Self> {
        let fields = split_fields(message, FIELD);
        let Some(&command) = fields.first() else {
            return Ok(Self::Unknown(message.to_owned()));
        };
        if let Some(kind) = TangentKind::from_command_name(command) {
            return parse_tangent(kind, &fields);
        }
        match command {
            TS_INFO => parse_ts_info(&fields),
            KILLED => parse_killed(&fields),
            SPEAKERS => parse_speakers(&fields),
            RELEASE_ALL_TANGENTS => parse_release_all_tangents(&fields),
            VERSION => parse_version(&fields),
            TRACK => parse_track(&fields),
            POS => parse_unit_position(&fields),
            FREQ => parse_frequencies(&fields),
            IS_SPEAKING => parse_is_speaking(&fields),
            _ => Ok(Self::Unknown(message.to_owned())),
        }
    }

    /// Whether this command counts as game activity, the signal that the player is in a mission:
    /// every command but `VERSION`, and an unknown message by the rule of
    /// [`message_counts_as_game_activity`] on its text. For a message that fails to parse, the
    /// caller applies that function to the raw text.
    pub fn counts_as_game_activity(&self) -> bool {
        match self {
            Self::Version { .. } => false,
            Self::Unknown(message) => message_counts_as_game_activity(message),
            _ => true,
        }
    }
}
