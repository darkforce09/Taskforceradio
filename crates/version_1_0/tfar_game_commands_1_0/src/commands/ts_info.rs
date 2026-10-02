//! `TsInfoQuery`: what a `TS_INFO` call asks about the TeamSpeak connection.

use crate::replies::{FAIL, PONG};

#[cfg(test)]
#[path = "tests/ts_info.rs"]
mod tests;

/// The query of a `TS_INFO` call, its second field matched exactly as `ts_info` matches it
/// (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:615-629`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TsInfoQuery {
    /// `SERVER`: the server name.
    Server,
    /// `SERVERUID`: the server's unique identifier.
    ServerUid,
    /// `CHANNEL`: the name of the local player's channel.
    Channel,
    /// `CHANNELID`: the id of the local player's channel, in decimal.
    ChannelId,
    /// `PING`: answered `PONG`.
    Ping,
    /// `VERSION`: the plugin version.
    Version,
    /// Any other query, kept as sent; answered `FAIL`.
    Other(String),
}

impl TsInfoQuery {
    /// The query `text` names.
    pub fn parse(text: &str) -> Self {
        match text {
            "SERVER" => Self::Server,
            "SERVERUID" => Self::ServerUid,
            "CHANNEL" => Self::Channel,
            "CHANNELID" => Self::ChannelId,
            "PING" => Self::Ping,
            "VERSION" => Self::Version,
            other => Self::Other(other.to_owned()),
        }
    }

    /// The answer that needs no TeamSpeak state: `PONG` for [`TsInfoQuery::Ping`], `FAIL` for
    /// [`TsInfoQuery::Other`]; `None` for the queries the plugin answers from TeamSpeak.
    pub const fn fixed_reply(&self) -> Option<&'static str> {
        match self {
            Self::Ping => Some(PONG),
            Self::Other(_) => Some(FAIL),
            Self::Server | Self::ServerUid | Self::Channel | Self::ChannelId | Self::Version => {
                None
            }
        }
    }
}
