//! The answers a plugin gives TeamSpeak, and the item an info panel asks about.
//!
//! **Role:** each answer is an enum with the SDK's integer value, so a plugin never returns a bare
//! `0` or `1` whose meaning flips between callbacks (`init` treats `0` as success,
//! `onServerErrorEvent` treats `1` as "handled").
//!
//! **Position:** inside `plugin_exports`; returned by [`Ts3Plugin`](crate::Ts3Plugin) methods and
//! turned into integers by the glue.
//!
//! **Signals and state:** plain values; no state.
//!
//! **Invariants:** `to_raw` gives the value `src/plugin.h` documents; [`InfoItem::from_raw`] maps
//! the `enum PluginItemType` values `src/tests/sdk_layout.rs` checks against the header.

use crate::{ChannelId, ClientId};

/// The answer of `ts3plugin_init`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitStatus {
    /// `0`: the plugin is running.
    Success,
    /// `1`: the plugin failed; TeamSpeak unloads it and shows its failure message.
    Failure,
    /// `2`: the plugin failed and has already told the user; TeamSpeak unloads it silently.
    FailureAlreadyReported,
}

impl InitStatus {
    /// The integer TeamSpeak expects.
    pub fn to_raw(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::Failure => 1,
            Self::FailureAlreadyReported => 2,
        }
    }
}

/// The answer of `ts3plugin_processCommand`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandHandling {
    /// `0`: the plugin handled the command.
    Handled,
    /// `1`: the plugin did not handle it; TeamSpeak reports an unknown command.
    NotHandled,
}

impl CommandHandling {
    /// The integer TeamSpeak expects.
    pub fn to_raw(self) -> i32 {
        match self {
            Self::Handled => 0,
            Self::NotHandled => 1,
        }
    }
}

/// The answer of `onServerErrorEvent` and `onServerPermissionErrorEvent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorDisplay {
    /// `0`: TeamSpeak shows the error as usual.
    Show,
    /// `1`: the plugin handled the error; TeamSpeak does not show it.
    Suppress,
}

impl ErrorDisplay {
    /// The integer TeamSpeak expects.
    pub fn to_raw(self) -> i32 {
        match self {
            Self::Show => 0,
            Self::Suppress => 1,
        }
    }
}

/// The item a `ts3plugin_infoData` call asks about (`enum PluginItemType` and its ID).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoItem {
    /// `PLUGIN_SERVER`: the server itself.
    Server,
    /// `PLUGIN_CHANNEL`: a channel.
    Channel(ChannelId),
    /// `PLUGIN_CLIENT`: a client.
    Client(ClientId),
}

impl InfoItem {
    /// The item for TeamSpeak's type and ID, or `None` for a type the SDK does not define or a
    /// client ID beyond `anyID`.
    pub fn from_raw(item_type: i32, id: u64) -> Option<Self> {
        match item_type {
            0 => Some(Self::Server),
            1 => Some(Self::Channel(ChannelId(id))),
            2 => u16::try_from(id)
                .ok()
                .map(|client| Self::Client(ClientId(client))),
            _ => None,
        }
    }
}
