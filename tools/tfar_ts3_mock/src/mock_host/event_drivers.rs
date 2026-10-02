//! The event drivers of [`MockHost`]: what the TeamSpeak server and the other clients do, played
//! into the plugin as the callbacks TeamSpeak raises.
//!
//! **Role:** `connect_server`, `disconnect_server`, `set_current_server`, `client_joins`,
//! `client_moves`, `client_leaves`, `set_talking`, `set_client_metadata` and
//! `deliver_plugin_command`: each changes the fake state, then calls the matching
//! `ts3plugin_on*` export.
//!
//! **Position:** an `impl MockHost` block beside `host_lifecycle`; called by tests on their own
//! thread, which stands in for TeamSpeak's main thread.
//!
//! **Signals and state:** the fake state, changed under its lock; the lock is released before the
//! plugin is called, since the plugin answers events by calling the fakes.
//!
//! **Invariants:** the state a callback's plugin reads already holds the change it announces; a
//! callback the plugin does not export is skipped (TeamSpeak would not call it); the string
//! arguments live until the callback returns.

use std::collections::BTreeMap;
use std::ffi::{CString, c_int};

use tfar_ts3_host::{ChannelId, ClientId, ConnectStatus, ServerConnectionHandlerId};
use ts3plugin_sys::public_definitions::Visibility;

use super::host_lifecycle::MockHost;
use crate::fake_state::host_session::with_fake_teamspeak;
use crate::{Error, FakeChannel, FakeClient, FakeServer, Result};

#[cfg(test)]
#[path = "tests/event_drivers.rs"]
mod tests;

/// The server tab [`MockHost::connect_server`] creates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerSetup {
    /// `VIRTUALSERVER_NAME`.
    pub name: String,
    /// `VIRTUALSERVER_UNIQUE_IDENTIFIER`.
    pub unique_identifier: String,
    /// Our nickname on the server.
    pub own_nickname: String,
    /// Our unique identifier.
    pub own_unique_identifier: String,
    /// The top-level channels, given IDs `1, 2, …` in order; we join the first. Empty means one
    /// channel called `Default Channel`.
    pub channel_names: Vec<String>,
}

impl Default for ServerSetup {
    fn default() -> Self {
        Self {
            name: "Mock Server".to_owned(),
            unique_identifier: "mock-server-unique-identifier=".to_owned(),
            own_nickname: "Mock Player".to_owned(),
            own_unique_identifier: "mock-player-unique-identifier=".to_owned(),
            channel_names: vec!["Default Channel".to_owned()],
        }
    }
}

/// A C string for the argument `argument` of a callback.
fn c_string(argument: &'static str, text: &str) -> Result<CString> {
    CString::new(text).map_err(|_| Error::InteriorNul { argument })
}

impl MockHost {
    /// Connects a new server tab as TeamSpeak does: makes it current
    /// (`currentServerConnectionChanged`), raises `onConnectStatusChangeEvent` for connecting,
    /// connected and establishing, `onNewChannelEvent` per channel, then established.
    ///
    /// # Errors
    ///
    /// [`Error::InteriorNul`] when a name holds a NUL byte.
    pub fn connect_server(&self, setup: &ServerSetup) -> Result<ServerConnectionHandlerId> {
        let channel_names = if setup.channel_names.is_empty() {
            ServerSetup::default().channel_names
        } else {
            setup.channel_names.clone()
        };
        for name in &channel_names {
            c_string("channel name", name)?;
        }
        let channels: BTreeMap<ChannelId, FakeChannel> = (1_u64..)
            .zip(channel_names)
            .map(|(id, name)| {
                let channel = FakeChannel {
                    name,
                    parent: ChannelId(0),
                };
                (ChannelId(id), channel)
            })
            .collect();
        let channel_ids: Vec<ChannelId> = channels.keys().copied().collect();
        let own_client = ClientId(1);
        let own = FakeClient::new(
            &setup.own_nickname,
            &setup.own_unique_identifier,
            channel_ids.first().copied().unwrap_or(ChannelId(1)),
        );
        let server = with_fake_teamspeak(|fake| {
            let highest = fake.servers.keys().next_back().map_or(0, |server| server.0);
            let server = ServerConnectionHandlerId(highest.saturating_add(1));
            fake.servers.insert(
                server,
                FakeServer {
                    name: setup.name.clone(),
                    unique_identifier: setup.unique_identifier.clone(),
                    connect_status: ConnectStatus::Disconnected,
                    own_client,
                    channels,
                    clients: BTreeMap::from([(own_client, own)]),
                },
            );
            fake.current_server = server;
            server
        });
        self.announce_current_server(server);
        for status in [
            ConnectStatus::Connecting,
            ConnectStatus::Connected,
            ConnectStatus::ConnectionEstablishing,
        ] {
            self.change_connect_status(server, status);
        }
        if let Some(on_new_channel) = self.exports().on_new_channel_event {
            for channel in channel_ids {
                // SAFETY: plain values; the plugin may query the channel, which exists.
                unsafe { on_new_channel(server.0, channel.0, 0) };
            }
        }
        self.change_connect_status(server, ConnectStatus::ConnectionEstablished);
        Ok(server)
    }

    /// Disconnects `server`: its channels and clients go, its status becomes disconnected and
    /// `onConnectStatusChangeEvent` says so.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownServer`] when the tab does not exist.
    pub fn disconnect_server(&self, server: ServerConnectionHandlerId) -> Result<()> {
        with_fake_teamspeak(|fake| {
            let fake_server = fake
                .server_mut(server)
                .ok_or(Error::UnknownServer { server })?;
            fake_server.channels.clear();
            fake_server.clients.clear();
            Ok(())
        })?;
        self.change_connect_status(server, ConnectStatus::Disconnected);
        Ok(())
    }

    /// Brings the tab `server` to the front (`currentServerConnectionChanged`).
    ///
    /// # Errors
    ///
    /// [`Error::UnknownServer`] when the tab does not exist.
    pub fn set_current_server(&self, server: ServerConnectionHandlerId) -> Result<()> {
        with_fake_teamspeak(|fake| {
            if fake.server(server).is_none() {
                return Err(Error::UnknownServer { server });
            }
            fake.current_server = server;
            Ok(())
        })?;
        self.announce_current_server(server);
        Ok(())
    }

    /// A new client called `nickname` joins the current server in `channel`
    /// (`onClientMoveEvent` from channel `0`, entering view).
    ///
    /// # Errors
    ///
    /// [`Error::NoCurrentServer`], [`Error::UnknownChannel`], [`Error::ClientIdsExhausted`].
    pub fn client_joins(&self, nickname: &str, channel: ChannelId) -> Result<ClientId> {
        let server = self.connected_server()?;
        let client = with_fake_teamspeak(|fake| {
            let fake_server = fake.server_mut(server).ok_or(Error::NoCurrentServer)?;
            if !fake_server.channels.contains_key(&channel) {
                return Err(Error::UnknownChannel { channel });
            }
            let client = fake_server
                .next_client_id()
                .ok_or(Error::ClientIdsExhausted { server })?;
            let unique_identifier = format!("mock-client-{}=", client.0);
            fake_server.clients.insert(
                client,
                FakeClient::new(nickname, &unique_identifier, channel),
            );
            Ok(client)
        })?;
        self.announce_client_move(server, client, ChannelId(0), channel, Visibility::Enter, "")?;
        Ok(client)
    }

    /// `client` (our own included) moves to `channel` on the current server
    /// (`onClientMoveEvent`, staying in view).
    ///
    /// # Errors
    ///
    /// [`Error::NoCurrentServer`], [`Error::UnknownClient`], [`Error::UnknownChannel`].
    pub fn client_moves(&self, client: ClientId, channel: ChannelId) -> Result<()> {
        let server = self.connected_server()?;
        let old_channel = with_fake_teamspeak(|fake| {
            let fake_server = fake.server_mut(server).ok_or(Error::NoCurrentServer)?;
            if !fake_server.channels.contains_key(&channel) {
                return Err(Error::UnknownChannel { channel });
            }
            let fake_client = fake_server
                .clients
                .get_mut(&client)
                .ok_or(Error::UnknownClient { client })?;
            Ok(std::mem::replace(&mut fake_client.channel, channel))
        })?;
        self.announce_client_move(server, client, old_channel, channel, Visibility::Retain, "")
    }

    /// `client` leaves the current server (`onClientMoveEvent` to channel `0`, leaving view).
    ///
    /// # Errors
    ///
    /// [`Error::NoCurrentServer`], [`Error::UnknownClient`].
    pub fn client_leaves(&self, client: ClientId) -> Result<()> {
        let server = self.connected_server()?;
        let old_channel = with_fake_teamspeak(|fake| {
            let fake_server = fake.server_mut(server).ok_or(Error::NoCurrentServer)?;
            let fake_client = fake_server
                .clients
                .remove(&client)
                .ok_or(Error::UnknownClient { client })?;
            Ok(fake_client.channel)
        })?;
        let visibility = Visibility::Leave;
        self.announce_client_move(
            server,
            client,
            old_channel,
            ChannelId(0),
            visibility,
            "leaving",
        )
    }

    /// Sets whether `client` on the current server is talking (`CLIENT_FLAG_TALKING`); TFAR
    /// reads the flag, so no callback is raised.
    ///
    /// # Errors
    ///
    /// [`Error::NoCurrentServer`], [`Error::UnknownClient`].
    pub fn set_talking(&self, client: ClientId, is_talking: bool) -> Result<()> {
        let server = self.connected_server()?;
        with_fake_teamspeak(|fake| {
            let fake_client = fake
                .client_mut(server, client)
                .ok_or(Error::UnknownClient { client })?;
            fake_client.is_talking = is_talking;
            Ok(())
        })
    }

    /// `client` on the current server publishes `metadata` (`CLIENT_META_DATA`), announced with
    /// `onUpdateClientEvent` invoked by the client itself.
    ///
    /// # Errors
    ///
    /// [`Error::NoCurrentServer`], [`Error::UnknownClient`], [`Error::InteriorNul`].
    pub fn set_client_metadata(&self, client: ClientId, metadata: &str) -> Result<()> {
        let server = self.connected_server()?;
        let (nickname, unique_identifier) = with_fake_teamspeak(|fake| {
            let fake_client = fake
                .client_mut(server, client)
                .ok_or(Error::UnknownClient { client })?;
            metadata.clone_into(&mut fake_client.metadata);
            Ok((
                fake_client.nickname.clone(),
                fake_client.unique_identifier.clone(),
            ))
        })?;
        let nickname = c_string("nickname", &nickname)?;
        let unique_identifier = c_string("unique identifier", &unique_identifier)?;
        if let Some(on_update_client) = self.exports().on_update_client_event {
            // SAFETY: the strings are NUL-terminated and outlive the call.
            unsafe {
                on_update_client(
                    server.0,
                    client.0,
                    client.0,
                    nickname.as_ptr(),
                    unique_identifier.as_ptr(),
                );
            };
        }
        Ok(())
    }

    /// `from_client` on the current server sends `command` to the plugins called `plugin_name`
    /// (`onPluginCommandEvent`).
    ///
    /// # Errors
    ///
    /// [`Error::NoCurrentServer`], [`Error::UnknownClient`], [`Error::InteriorNul`].
    pub fn deliver_plugin_command(
        &self,
        from_client: ClientId,
        plugin_name: &str,
        command: &str,
    ) -> Result<()> {
        let server = self.connected_server()?;
        let (nickname, unique_identifier) = with_fake_teamspeak(|fake| {
            let fake_client = fake
                .client(server, from_client)
                .ok_or(Error::UnknownClient {
                    client: from_client,
                })?;
            Ok((
                fake_client.nickname.clone(),
                fake_client.unique_identifier.clone(),
            ))
        })?;
        let plugin_name = c_string("plugin name", plugin_name)?;
        let command = c_string("command", command)?;
        let nickname = c_string("nickname", &nickname)?;
        let unique_identifier = c_string("unique identifier", &unique_identifier)?;
        if let Some(on_plugin_command) = self.exports().on_plugin_command_event {
            // SAFETY: the strings are NUL-terminated and outlive the call.
            unsafe {
                on_plugin_command(
                    server.0,
                    plugin_name.as_ptr(),
                    command.as_ptr(),
                    from_client.0,
                    nickname.as_ptr(),
                    unique_identifier.as_ptr(),
                );
            };
        }
        Ok(())
    }

    /// The current server, when one is connected.
    fn connected_server(&self) -> Result<ServerConnectionHandlerId> {
        let server = self.current_server();
        let is_known = with_fake_teamspeak(|fake| fake.server(server).is_some());
        if is_known {
            Ok(server)
        } else {
            Err(Error::NoCurrentServer)
        }
    }

    /// Calls `currentServerConnectionChanged(server)`.
    fn announce_current_server(&self, server: ServerConnectionHandlerId) {
        if let Some(changed) = self.exports().current_server_connection_changed {
            // SAFETY: a plain value.
            unsafe { changed(server.0) };
        }
    }

    /// Sets the status of `server` and raises `onConnectStatusChangeEvent`.
    fn change_connect_status(&self, server: ServerConnectionHandlerId, status: ConnectStatus) {
        with_fake_teamspeak(|fake| {
            if let Some(fake_server) = fake.server_mut(server) {
                fake_server.connect_status = status;
            }
        });
        if let Some(on_status) = self.exports().on_connect_status_change_event {
            // SAFETY: plain values; error `0` is `ERROR_ok`.
            unsafe { on_status(server.0, status.to_raw(), 0) };
        }
    }

    /// Raises `onClientMoveEvent`.
    fn announce_client_move(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
        old_channel: ChannelId,
        new_channel: ChannelId,
        visibility: Visibility,
        message: &str,
    ) -> Result<()> {
        let message = c_string("move message", message)?;
        if let Some(on_client_move) = self.exports().on_client_move_event {
            // SAFETY: plain values and a NUL-terminated message that outlives the call.
            unsafe {
                on_client_move(
                    server.0,
                    client.0,
                    old_channel.0,
                    new_channel.0,
                    visibility as c_int,
                    message.as_ptr(),
                );
            };
        }
        Ok(())
    }
}
