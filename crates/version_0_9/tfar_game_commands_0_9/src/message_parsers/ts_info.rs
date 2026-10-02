//! `TS_INFO`: the game asks the plugin about TeamSpeak.

use crate::command_names::TS_INFO;
use crate::error::Result;
use crate::field_values::exact_fields;
use crate::game_command::GameCommand;

#[cfg(test)]
#[path = "tests/ts_info.rs"]
mod tests;

/// What a `TS_INFO` message asks, as `ts_info` in `reference/tfar_0_9_12/ts/src/plugin.cpp` reads
/// its second field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TsInfoQuery {
    /// `SERVER`: the name of the TeamSpeak server.
    Server,
    /// `CHANNEL`: the name of the player's channel.
    Channel,
    /// `PING`: a liveness check, answered `PONG`.
    Ping,
    /// Any other query, kept as sent; answered `FAIL`.
    Other(String),
}

/// `TS_INFO\t<query>`: exactly 2 fields.
pub(crate) fn parse_ts_info(fields: &[&str]) -> Result<GameCommand> {
    let [_, query] = exact_fields::<2>(TS_INFO, fields)?;
    Ok(GameCommand::TsInfo(match query {
        "SERVER" => TsInfoQuery::Server,
        "CHANNEL" => TsInfoQuery::Channel,
        "PING" => TsInfoQuery::Ping,
        other => TsInfoQuery::Other(other.to_owned()),
    }))
}
