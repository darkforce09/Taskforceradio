//! `CommandName`: the name in the first field of every game message the 1.0 plugin knows.
//!
//! **Role:** map the first field of a message to the command it names, by exact name, and say
//! which commands answer only in the async queue.
//!
//! **Position:** a module of `tfar_game_commands_1_0`; used by the parser, the errors and the
//! plugin's sync dispatch.
//!
//! **Signals and state:** none; a `Copy` enum.
//!
//! **Invariants:** the names are those of `toGameCommand`
//! (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:165-205`), matched exactly in place of its
//! FNV-1a hash switch; `RECV_FREQS` is not among them, as the original never maps it.

#[cfg(test)]
#[path = "tests/command_name.rs"]
mod tests;

/// A game message command the 1.0 plugin knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CommandName {
    /// `TS_INFO`: a query about the TeamSpeak connection.
    TsInfo,
    /// `POS`: one player's position and state.
    Position,
    /// `IS_SPEAKING`: whether one player speaks.
    IsSpeaking,
    /// `IS_SPEAKING_BULK`: whether each of several players speaks.
    IsSpeakingBulk,
    /// `FREQ`: the local player's radios and state.
    Frequencies,
    /// `KILLED`: a player died.
    Killed,
    /// `TRACK`: a usage-statistics event.
    Track,
    /// `DFRAME`: the game started a new data frame.
    DataFrame,
    /// `SPEAKERS`: the radios playing over loudspeakers.
    Speakers,
    /// `TANGENT`: a short-range radio press or release.
    Tangent,
    /// `TANGENT_LR`: a long-range radio press or release.
    TangentLongRange,
    /// `TANGENT_DD`: a diver radio press or release.
    TangentDiverDigital,
    /// `RELEASE_ALL_TANGENTS`: every radio of a player released.
    ReleaseAllTangents,
    /// `SETCFG`: one plugin setting.
    SetConfig,
    /// `MISSIONEND`: the mission ended.
    MissionEnd,
    /// `RadioTwrAdd`: radio towers added.
    RadioTowerAdd,
    /// `RadioTwrDel`: radio towers removed.
    RadioTowerRemove,
    /// `collectDebugInfo`: the player asked for a diagnostics dump.
    CollectDebugInfo,
}

impl CommandName {
    /// Every command name.
    pub const ALL: [Self; 18] = [
        Self::TsInfo,
        Self::Position,
        Self::IsSpeaking,
        Self::IsSpeakingBulk,
        Self::Frequencies,
        Self::Killed,
        Self::Track,
        Self::DataFrame,
        Self::Speakers,
        Self::Tangent,
        Self::TangentLongRange,
        Self::TangentDiverDigital,
        Self::ReleaseAllTangents,
        Self::SetConfig,
        Self::MissionEnd,
        Self::RadioTowerAdd,
        Self::RadioTowerRemove,
        Self::CollectDebugInfo,
    ];

    /// The command's name on the wire, letter case included.
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::TsInfo => "TS_INFO",
            Self::Position => "POS",
            Self::IsSpeaking => "IS_SPEAKING",
            Self::IsSpeakingBulk => "IS_SPEAKING_BULK",
            Self::Frequencies => "FREQ",
            Self::Killed => "KILLED",
            Self::Track => "TRACK",
            Self::DataFrame => "DFRAME",
            Self::Speakers => "SPEAKERS",
            Self::Tangent => "TANGENT",
            Self::TangentLongRange => "TANGENT_LR",
            Self::TangentDiverDigital => "TANGENT_DD",
            Self::ReleaseAllTangents => "RELEASE_ALL_TANGENTS",
            Self::SetConfig => "SETCFG",
            Self::MissionEnd => "MISSIONEND",
            Self::RadioTowerAdd => "RadioTwrAdd",
            Self::RadioTowerRemove => "RadioTwrDel",
            Self::CollectDebugInfo => "collectDebugInfo",
        }
    }

    /// The command whose wire name is exactly `name`; `None` for any other text.
    pub fn from_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|command| command.wire_name() == name)
    }

    /// Whether the command works only from the async queue. A sync call of such a command is
    /// answered `ASYNC COMMAND SENT IN SYNC CONTEXT` without being run
    /// (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:161`). `TS_INFO`, `IS_SPEAKING` and
    /// `IS_SPEAKING_BULK` answer sync; `POS` answers sync and is also processed in the queue
    /// (`:85-88`); every other command is async only.
    pub const fn is_async_only(self) -> bool {
        !matches!(
            self,
            Self::TsInfo | Self::Position | Self::IsSpeaking | Self::IsSpeakingBulk
        )
    }
}
