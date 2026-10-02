//! `ClientId`: the id TeamSpeak gives a client on one server connection.

/// A TeamSpeak client id on one server connection (`anyID` in the SDK).
///
/// TeamSpeak hands out ids from 1; 0 names no client, and the plugin never mutes or unmutes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClientId(pub u16);

impl ClientId {
    /// The id TeamSpeak's `anyID` value `value` names.
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// The raw `anyID` value.
    pub const fn get(self) -> u16 {
        self.0
    }

    /// Whether the id can name a client: TeamSpeak never gives out 0, which the original's
    /// `setClientMuteStatus` skips (`reference/tfar_0_9_12/ts/src/plugin.cpp:913`).
    pub const fn names_a_client(self) -> bool {
        self.0 != 0
    }
}
