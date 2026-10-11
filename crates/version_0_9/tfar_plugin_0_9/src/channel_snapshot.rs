//! The local player's channel as TeamSpeak reports it, read in one pass, and what the audio
//! callbacks keep of it.
//!
//! **Role:** read the channel client list with each client's nickname and metadata
//! (`updateNicknamesList`, `reference/tfar_0_9_12/ts/src/plugin.cpp:1478`), and keep the facts
//! the voice callbacks need but may not ask TeamSpeak for: our client id, each client's channel
//! name and whether their metadata shows a playing TFAR plugin.
//!
//! **Position:** read by `read_channel_snapshot` on the command threads (TeamSpeak events, the
//! game link thread, the service sweep), applied under the server lock by
//! `ServerState::apply_channel_snapshot`; `TeamSpeakFacts` is then read by the audio callbacks.
//!
//! **Signals and state:** `TeamSpeakFacts` lives inside the server lock; the snapshot is a
//! plain value made outside any lock.
//!
//! **Invariants:** the read makes every TeamSpeak call before any lock is taken; a client the
//! facts do not know has no channel name and no playing plugin, so its voice passes through.

use std::collections::BTreeMap;

use tfar_plugin_logic_0_9::client_state::ClientId as StateClientId;
use tfar_plugin_logic_0_9::plugin_commands::metadata_says_playing;
use tfar_ts3_host::{ChannelId, ClientId, ServerConnectionHandlerId};

use crate::team_speak::TeamSpeakQueries;

#[cfg(test)]
#[path = "tests/channel_snapshot.rs"]
mod tests;

/// The `tfar_plugin_logic_0_9::client_state` id of the TeamSpeak client `client`.
pub const fn state_client_id(client: ClientId) -> StateClientId {
    StateClientId::new(client.0)
}

/// The TeamSpeak id of the `tfar_plugin_logic_0_9::client_state` client `client`.
pub const fn host_client_id(client: StateClientId) -> ClientId {
    ClientId(client.get())
}

/// One client of the local player's channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelMember {
    /// The client's id.
    pub client: ClientId,
    /// The client's nickname.
    pub nickname: String,
    /// Whether the client's metadata shows a TFAR plugin connected to a running game.
    pub metadata_shows_plugin: bool,
}

/// The local player's channel, read from TeamSpeak in one pass.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChannelSnapshot {
    /// Our own client id; `None` when TeamSpeak cannot tell it (not connected).
    pub my_client: Option<ClientId>,
    /// Our own nickname; empty when unknown.
    pub my_nickname: String,
    /// The channel we sit in.
    pub channel: Option<ChannelId>,
    /// That channel's name; empty when unknown.
    pub channel_name: String,
    /// Every client of the channel, ourselves included.
    pub members: Vec<ChannelMember>,
}

/// Reads the local player's channel on `server`. Makes TeamSpeak calls: never under a lock.
pub fn read_channel_snapshot(
    team_speak: &dyn TeamSpeakQueries,
    server: ServerConnectionHandlerId,
) -> ChannelSnapshot {
    let Some(my_client) = team_speak.my_client_id(server) else {
        return ChannelSnapshot::default();
    };
    let my_nickname = team_speak
        .client_nickname(server, my_client)
        .unwrap_or_default();
    let channel = team_speak.channel_of_client(server, my_client);
    let channel_name = channel
        .and_then(|channel| team_speak.channel_name(server, channel))
        .unwrap_or_default();
    let members = channel
        .map(|channel| team_speak.channel_clients(server, channel))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|client| {
            let nickname = team_speak.client_nickname(server, client)?;
            let metadata_shows_plugin = team_speak
                .client_metadata(server, client)
                .is_some_and(|metadata| metadata_says_playing(&metadata));
            Some(ChannelMember {
                client,
                nickname,
                metadata_shows_plugin,
            })
        })
        .collect();
    ChannelSnapshot {
        my_client: Some(my_client),
        my_nickname,
        channel,
        channel_name,
        members,
    }
}

impl ChannelSnapshot {
    /// The `(client id, nickname)` pairs `ServerRadioData::update_nicknames` takes.
    pub fn nickname_pairs(&self) -> Vec<(StateClientId, String)> {
        self.members
            .iter()
            .map(|member| (state_client_id(member.client), member.nickname.clone()))
            .collect()
    }

    /// The members' client ids, as `tfar_plugin_logic_0_9::client_state` names them.
    pub fn member_ids(&self) -> Vec<StateClientId> {
        self.members
            .iter()
            .map(|member| state_client_id(member.client))
            .collect()
    }
}

/// What one client's voice frame needs to know from TeamSpeak.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClientFacts {
    /// Whether the client's metadata shows a playing TFAR plugin.
    pub metadata_shows_plugin: bool,
    /// The name of the channel the client sits in.
    pub channel_name: String,
}

/// The TeamSpeak facts the voice callbacks read under the server lock instead of calling
/// TeamSpeak on the audio thread.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TeamSpeakFacts {
    /// Our own client id on the connection.
    pub my_client: Option<StateClientId>,
    clients: BTreeMap<StateClientId, ClientFacts>,
}

impl TeamSpeakFacts {
    /// Replaces every fact with those of `snapshot`.
    pub fn replace_with(&mut self, snapshot: &ChannelSnapshot) {
        self.my_client = snapshot.my_client.map(state_client_id);
        self.clients.clear();
        for member in &snapshot.members {
            self.clients.insert(
                state_client_id(member.client),
                ClientFacts {
                    metadata_shows_plugin: member.metadata_shows_plugin,
                    channel_name: snapshot.channel_name.clone(),
                },
            );
        }
    }

    /// Forgets `client`, which left.
    pub fn forget(&mut self, client: StateClientId) {
        self.clients.remove(&client);
    }

    /// The facts of `client`, if known.
    pub fn client(&self, client: StateClientId) -> Option<&ClientFacts> {
        self.clients.get(&client)
    }

    /// Whether `client`'s metadata shows a playing plugin; `false` when unknown.
    pub fn metadata_shows_plugin(&self, client: StateClientId) -> bool {
        self.client(client)
            .is_some_and(|facts| facts.metadata_shows_plugin)
    }

    /// The name of `client`'s channel; empty when unknown.
    pub fn channel_name_of(&self, client: StateClientId) -> &str {
        self.client(client)
            .map_or("", |facts| facts.channel_name.as_str())
    }
}
