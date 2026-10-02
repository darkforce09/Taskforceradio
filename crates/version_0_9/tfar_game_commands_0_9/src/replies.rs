//! The reply strings the 0.9.12 plugin sends back for every game message.
//!
//! **Role:** name each reply string once, and give the reply a command earns when the plugin has
//! nothing message-specific to say.
//!
//! **Position:** a module of `tfar_game_commands_0_9`; the plugin app answers the game with these
//! strings, and `Error::reply` uses `FAIL`.
//!
//! **Signals and state:** none; constants and a pure function.
//!
//! **Invariants:** every string is the one `processGameCommand` and `ts_info` return in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp`, byte for byte; `KILLED` replies `DONE`, and
//! `RELEASE_ALL_TANGENTS` and `TRACK` reply `OK` rather than falling through to `FAIL`.

use crate::game_command::GameCommand;
use crate::message_parsers::TsInfoQuery;

#[cfg(test)]
#[path = "tests/replies.rs"]
mod tests;

/// The reply to a message the plugin handled.
pub const OK: &str = "OK";

/// The reply to a message the plugin does not handle or could not parse.
pub const FAIL: &str = "FAIL";

/// The reply to `KILLED`.
pub const DONE: &str = "DONE";

/// The reply to `TS_INFO PING`.
pub const PONG: &str = "PONG";

/// The reply to `POS` and `IS_SPEAKING` when the named player is talking, directly or over a radio.
pub const SPEAKING: &str = "SPEAKING";

/// The reply to `POS` and `IS_SPEAKING` when the named player is silent or unknown.
pub const NOT_SPEAKING: &str = "NOT_SPEAKING";

/// The reply to `TS_INFO SERVER` when TeamSpeak cannot name the server.
pub const ERROR_GETTING_SERVER_NAME: &str = "ERROR_GETTING_SERVER_NAME";

/// The reply to `TS_INFO CHANNEL` when TeamSpeak cannot name the channel: the empty string.
pub const NO_CHANNEL_NAME: &str = "";

/// The reply `command` earns from its kind alone, before any TeamSpeak lookup: the fixed reply of
/// a command whose reply never varies, and the fallback the original sends when its lookup finds
/// nothing for the three that vary (the server name, the channel name, the speaking state).
///
/// | Command | Reply |
/// |---|---|
/// | `TS_INFO SERVER` | the server name; [`ERROR_GETTING_SERVER_NAME`] without one |
/// | `TS_INFO CHANNEL` | the channel name; [`NO_CHANNEL_NAME`] without one |
/// | `TS_INFO PING` | [`PONG`] |
/// | `TS_INFO` other | [`FAIL`] |
/// | `KILLED` | [`DONE`] |
/// | `SPEAKERS`, `VERSION`, `TANGENT*`, `FREQ` | [`OK`] |
/// | `RELEASE_ALL_TANGENTS`, `TRACK` | [`OK`] |
/// | `POS`, `IS_SPEAKING` | [`SPEAKING`] when talking; [`NOT_SPEAKING`] otherwise |
/// | anything else | [`FAIL`] |
pub fn reply_for_unhandled(command: &GameCommand) -> &'static str {
    match command {
        GameCommand::TsInfo(TsInfoQuery::Server) => ERROR_GETTING_SERVER_NAME,
        GameCommand::TsInfo(TsInfoQuery::Channel) => NO_CHANNEL_NAME,
        GameCommand::TsInfo(TsInfoQuery::Ping) => PONG,
        GameCommand::TsInfo(TsInfoQuery::Other(_)) | GameCommand::Unknown(_) => FAIL,
        GameCommand::Killed { .. } => DONE,
        GameCommand::Speakers(_)
        | GameCommand::ReleaseAllTangents { .. }
        | GameCommand::Version { .. }
        | GameCommand::Track { .. }
        | GameCommand::Tangent { .. }
        | GameCommand::Frequencies(_) => OK,
        GameCommand::Position(_) | GameCommand::IsSpeaking { .. } => NOT_SPEAKING,
    }
}
