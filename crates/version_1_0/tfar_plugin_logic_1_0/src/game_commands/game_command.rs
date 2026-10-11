//! `GameCommand`: one message from the game, parsed into its typed form.
//!
//! **Role:** the typed form of every 1.0 game message, with the sync rules the plugin's
//! dispatch needs.
//!
//! **Position:** a module of `tfar_plugin_logic_1_0::game_commands`; built by `parse`; matched by the plugin's
//! sync answer and its async queue worker.
//!
//! **Signals and state:** none; a value type.
//!
//! **Invariants:** one variant per command of `toGameCommand`
//! (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:165-205`) plus `Unknown`; the async-only
//! rule is [`CommandName::is_async_only`].

use crate::game_commands::command_name::CommandName;
use crate::game_commands::commands::{
    ConfigAssignment, FrequencyReport, PositionUpdate, SpeakerRecord, TangentPress, TowerRecord,
    TrackingEvent, TsInfoQuery,
};
use crate::game_commands::fields::NetworkObjectId;
use crate::game_commands::replies::{ASYNC_COMMAND_SENT_IN_SYNC_CONTEXT, UNKNOWN_COMMAND};

#[cfg(test)]
#[path = "tests/game_command.rs"]
mod tests;

/// One game message, parsed.
#[derive(Clone, Debug, PartialEq)]
pub enum GameCommand {
    /// `TS_INFO`: a query about the TeamSpeak connection.
    TsInfo(TsInfoQuery),
    /// `POS`: one player's position and state.
    Position(PositionUpdate),
    /// `IS_SPEAKING`: whether one player speaks.
    IsSpeaking {
        /// The player's nickname, sanitized.
        nickname: String,
    },
    /// `IS_SPEAKING_BULK`: whether each player speaks, in this order; each nickname sanitized.
    IsSpeakingBulk(Vec<String>),
    /// `FREQ`: the local player's radios and state.
    Frequencies(FrequencyReport),
    /// `KILLED`: a player died.
    Killed {
        /// The player's nickname, sanitized.
        nickname: String,
    },
    /// `DFRAME`: the game started a new data frame.
    DataFrame,
    /// `SPEAKERS`: every radio playing over a loudspeaker; the list replaces the previous one.
    Speakers(Vec<SpeakerRecord>),
    /// `TANGENT`, `TANGENT_LR` or `TANGENT_DD`: a radio press or release.
    Tangent(TangentPress),
    /// `RELEASE_ALL_TANGENTS`: every radio of a player released.
    ReleaseAllTangents {
        /// The player's nickname, sanitized.
        nickname: String,
    },
    /// `SETCFG`: one plugin setting.
    SetConfig(ConfigAssignment),
    /// `MISSIONEND`: the mission ended; the plugin does nothing, the game link's disconnect does
    /// the work.
    MissionEnd,
    /// `RadioTwrAdd`: radio towers added or replaced.
    RadioTowerAdd(Vec<TowerRecord>),
    /// `RadioTwrDel`: radio towers removed.
    RadioTowerRemove(Vec<NetworkObjectId>),
    /// `TRACK`: a usage-statistics event.
    Track(TrackingEvent),
    /// `collectDebugInfo`: the player asked for a diagnostics dump.
    CollectDebugInfo,
    /// A name the plugin does not know; the async queue ignores it.
    Unknown {
        /// The first field as sent.
        name: String,
    },
}

impl GameCommand {
    /// The command's name; `None` for [`GameCommand::Unknown`].
    pub const fn name(&self) -> Option<CommandName> {
        Some(match self {
            Self::TsInfo(_) => CommandName::TsInfo,
            Self::Position(_) => CommandName::Position,
            Self::IsSpeaking { .. } => CommandName::IsSpeaking,
            Self::IsSpeakingBulk(_) => CommandName::IsSpeakingBulk,
            Self::Frequencies(_) => CommandName::Frequencies,
            Self::Killed { .. } => CommandName::Killed,
            Self::DataFrame => CommandName::DataFrame,
            Self::Speakers(_) => CommandName::Speakers,
            Self::Tangent(press) => press.kind.command_name(),
            Self::ReleaseAllTangents { .. } => CommandName::ReleaseAllTangents,
            Self::SetConfig(_) => CommandName::SetConfig,
            Self::MissionEnd => CommandName::MissionEnd,
            Self::RadioTowerAdd(_) => CommandName::RadioTowerAdd,
            Self::RadioTowerRemove(_) => CommandName::RadioTowerRemove,
            Self::Track(_) => CommandName::Track,
            Self::CollectDebugInfo => CommandName::CollectDebugInfo,
            Self::Unknown { .. } => return None,
        })
    }

    /// Whether the command works only from the async queue ([`CommandName::is_async_only`]);
    /// false for [`GameCommand::Unknown`], which neither path runs.
    pub const fn is_async_only(&self) -> bool {
        match self.name() {
            Some(name) => name.is_async_only(),
            None => false,
        }
    }

    /// The answer to a sync call that needs no plugin state: `UNKNOWN COMMAND` for an unknown
    /// name, `ASYNC COMMAND SENT IN SYNC CONTEXT` for an async-only command, and the fixed
    /// `TS_INFO` answers (`PONG`, `FAIL`); `None` when the plugin must answer from its state.
    pub const fn fixed_sync_reply(&self) -> Option<&'static str> {
        match self {
            Self::Unknown { .. } => Some(UNKNOWN_COMMAND),
            Self::TsInfo(query) => query.fixed_reply(),
            Self::Position(_) | Self::IsSpeaking { .. } | Self::IsSpeakingBulk(_) => None,
            _ => Some(ASYNC_COMMAND_SENT_IN_SYNC_CONTEXT),
        }
    }
}
