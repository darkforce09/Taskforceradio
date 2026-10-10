//! Who is on the server: the connection's own client, the clients of the own channel, their
//! nicknames and metadata, kept in step with TeamSpeak's events.
//!
//! **Role:** `_onConnectStatusChangeEvent`, `_onChannelSwitchedEvent`, `_onClientMoved` and the
//! client update handling of `reference/tfar_1_0/ts/src/Teamspeak.cpp:294-370`, writing the client
//! directory of `tfar_client_state_1_0` and the metadata cache the voice gate reads; a client that
//! starts talking gets fresh direction sources.
//!
//! **Position:** methods of `PluginRuntime`; called from the TeamSpeak event callbacks on
//! TeamSpeak's main thread and, for the metadata, by the service thread.
//!
//! **Signals and state:** the server registry and each server state, written under their locks;
//! every TeamSpeak query runs before the lock is taken.
//!
//! **Invariants:** the directory holds the own client and the clients of the own channel only, as
//! TeamSpeak plays only the own channel; a client that leaves is unmuted if this plugin muted it.

use std::time::Instant;

use tfar_ts3_host::{ChannelId, ClientId, ClientMove, ServerConnectionHandlerId};

use crate::runtime::plugin_runtime::PluginRuntime;
use crate::runtime::server_state::state_client_id;

impl PluginRuntime {
    /// The connection to `server` is established: TeamSpeak's own 3D goes neutral, the state is
    /// reset, the channel cache and the client list of the own channel are read; a game in
    /// progress moves back into the serious channel.
    pub fn server_established(&self, server: ServerConnectionHandlerId) {
        let ts = self.ts();
        ts.neutralise_3d_settings(server);
        self.servers.remove(server);
        let state = self.server_state(server);
        if let Some(own) = ts.own_client_id(server) {
            let nickname = ts.client_nickname(server, own).unwrap_or_default();
            state
                .lock()
                .session
                .clients
                .set_own_client(state_client_id(own), &nickname);
            if let Some(channel) = ts.channel_of_client(server, own) {
                self.own_channel_switched(server, channel);
            }
        }
        self.refresh_channels(server);
        if self.is_in_game() {
            self.move_to_serious_channel(server, true);
        }
    }

    /// The connection to `server` closed: its state goes.
    pub fn server_disconnected(&self, server: ServerConnectionHandlerId) {
        self.servers.remove(server);
    }

    /// The local client entered `channel`: every other client is forgotten and the clients of the
    /// new channel join; serious mode is checked again.
    pub fn own_channel_switched(&self, server: ServerConnectionHandlerId, channel: ChannelId) {
        let ts = self.ts();
        let own = ts.own_client_id(server);
        let joined: Vec<(ClientId, String, String)> = ts
            .channel_clients(server, channel)
            .into_iter()
            .filter(|client| Some(*client) != own)
            .filter_map(|client| {
                let nickname = ts.client_nickname(server, client)?;
                let metadata = ts.client_metadata(server, client).unwrap_or_default();
                (!nickname.is_empty()).then_some((client, nickname, metadata))
            })
            .collect();
        let unmute = {
            let state = self.server_state(server);
            let mut state = state.lock();
            state.own_channel = Some(channel);
            state.session.clients.every_client_left();
            state.client_metadata.clear();
            let own_id = own.map(state_client_id);
            let leaving: Vec<_> = state
                .mutes
                .muted()
                .filter(|client| Some(*client) != own_id)
                .collect();
            for client in &leaving {
                state.mutes.client_left(*client);
            }
            for (client, nickname, metadata) in joined {
                let id = state_client_id(client);
                state.session.clients.client_joined(id, &nickname);
                state.client_metadata.insert(id, metadata);
            }
            leaving
        };
        self.send_mute_changes(server, unmute.into_iter().map(|client| (client, false)));
        self.check_serious_mode(server);
    }

    /// A client moved, timed out, was kicked or banned.
    pub fn client_moved(&self, movement: &ClientMove<'_>) {
        let server = movement.server;
        let ts = self.ts();
        if ts.own_client_id(server) == Some(movement.client) {
            if movement.new_channel.0 != 0 {
                self.own_channel_switched(server, movement.new_channel);
            }
            return;
        }
        let Some(state) = self.servers.get(server) else {
            return;
        };
        let own_channel = state.lock().own_channel;
        if own_channel.is_some() && Some(movement.new_channel) == own_channel {
            let nickname = ts.client_nickname(server, movement.client);
            let metadata = ts
                .client_metadata(server, movement.client)
                .unwrap_or_default();
            if let Some(nickname) = nickname.filter(|name| !name.is_empty()) {
                let mut state = state.lock();
                let id = state_client_id(movement.client);
                state.session.clients.client_joined(id, &nickname);
                state.client_metadata.insert(id, metadata);
            }
        } else {
            self.forget_client(server, movement.client);
        }
    }

    /// A client's variables changed: its nickname and metadata are read again.
    pub fn client_updated(&self, server: ServerConnectionHandlerId, client: ClientId) {
        let Some(state) = self.servers.get(server) else {
            return;
        };
        let ts = self.ts();
        let nickname = ts.client_nickname(server, client);
        let metadata = ts.client_metadata(server, client);
        let mut state = state.lock();
        let id = state_client_id(client);
        if let Some(nickname) = nickname.filter(|name| !name.is_empty()) {
            state.session.clients.client_renamed(id, &nickname);
        }
        if let Some(metadata) = metadata
            && state.session.clients.by_id(id).is_some()
        {
            state.client_metadata.insert(id, metadata);
        }
    }

    /// Reads the metadata of every known client of `server` again, and lets each read decide
    /// the plugin-enabled cache once it has expired.
    pub fn refresh_client_metadata(&self, server: ServerConnectionHandlerId) {
        let Some(state) = self.servers.get(server) else {
            return;
        };
        let clients: Vec<_> = state
            .lock()
            .session
            .clients
            .iter()
            .map(tfar_client_state_1_0::prelude::ClientData::client_id)
            .collect();
        let ts = self.ts();
        let read: Vec<_> = clients
            .into_iter()
            .filter_map(|id| {
                ts.client_metadata(server, crate::runtime::server_state::host_client_id(id))
                    .map(|metadata| (id, metadata))
            })
            .collect();
        let now = Instant::now();
        let mut state = state.lock();
        for (id, metadata) in read {
            if state.session.needs_plugin_metadata(id, now) {
                state.session.plugin_enabled_for(id, Some(&metadata), now);
            }
            state.client_metadata.insert(id, metadata);
        }
    }
}
