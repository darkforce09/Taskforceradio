//! [`FakeTeamSpeak`]: the servers, channels, clients, settings and paths the fakes answer from,
//! plus the call log and the allocation ledger.
//!
//! **Role:** the model of a TeamSpeak client the fakes read and the drivers change: what a real
//! client would know about its server tabs, and what the plugin asked of it.
//!
//! **Position:** owned by the process-global mutex in `host_session`; read and written by the
//! `extern "C"` fakes (on whichever thread the plugin calls from) and by the
//! [`MockHost`](crate::MockHost) drivers (on the test's thread).
//!
//! **Signals and state:** every field sits behind that one mutex; nothing here calls the plugin,
//! so no plugin code ever runs under the lock.
//!
//! **Invariants:** client and channel IDs are unique per server and never `0` (the SDK's list
//! terminator and "no channel"); the current server is `0` until a server connects.

use std::collections::BTreeMap;
use std::path::PathBuf;

use tfar_ts3_host::{ChannelId, ClientId, ConnectStatus, ServerConnectionHandlerId};

use super::allocation_ledger::AllocationLedger;
use crate::HostCall;

/// The fake TeamSpeak client: its server tabs, settings, paths and records.
#[derive(Debug)]
pub struct FakeTeamSpeak {
    /// The server tabs by handler ID.
    pub servers: BTreeMap<ServerConnectionHandlerId, FakeServer>,
    /// The tab in front; `0` when none.
    pub current_server: ServerConnectionHandlerId,
    /// The capture pre-processor settings (`vad` and the rest), shared by every tab.
    pub preprocessor_values: BTreeMap<String, String>,
    /// The folder `getPluginPath` answers with (written with a trailing `/`).
    pub plugin_path: PathBuf,
    /// The folder `getConfigPath` answers with (written with a trailing `/`).
    pub config_path: PathBuf,
    /// Every call the plugin made, in order.
    pub(crate) calls: Vec<HostCall>,
    /// The memory the fakes handed out and the plugin freed.
    pub(crate) ledger: AllocationLedger,
}

/// One server tab of the fake client.
#[derive(Debug, Clone, PartialEq)]
pub struct FakeServer {
    /// `VIRTUALSERVER_NAME`.
    pub name: String,
    /// `VIRTUALSERVER_UNIQUE_IDENTIFIER`.
    pub unique_identifier: String,
    /// What `getConnectionStatus` answers.
    pub connect_status: ConnectStatus,
    /// Our own client on this server.
    pub own_client: ClientId,
    /// The channels by ID.
    pub channels: BTreeMap<ChannelId, FakeChannel>,
    /// The visible clients by ID, our own included.
    pub clients: BTreeMap<ClientId, FakeClient>,
}

/// One channel of a fake server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeChannel {
    /// `CHANNEL_NAME`.
    pub name: String,
    /// The parent channel; `0` for a top-level channel.
    pub parent: ChannelId,
}

/// One client on a fake server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeClient {
    /// `CLIENT_NICKNAME`.
    pub nickname: String,
    /// `CLIENT_UNIQUE_IDENTIFIER`.
    pub unique_identifier: String,
    /// The channel the client is in.
    pub channel: ChannelId,
    /// `CLIENT_META_DATA`.
    pub metadata: String,
    /// `CLIENT_FLAG_TALKING`.
    pub is_talking: bool,
    /// `CLIENT_INPUT_MUTED`.
    pub input_muted: i32,
    /// `CLIENT_OUTPUT_MUTED`.
    pub output_muted: i32,
    /// `CLIENT_INPUT_DEACTIVATED` (push-to-talk released).
    pub input_deactivated: i32,
    /// `CLIENT_IS_MUTED`: muted locally through `requestMuteClients`.
    pub is_muted: bool,
}

impl FakeClient {
    /// A silent, unmuted client called `nickname` in `channel`.
    pub fn new(nickname: &str, unique_identifier: &str, channel: ChannelId) -> Self {
        Self {
            nickname: nickname.to_owned(),
            unique_identifier: unique_identifier.to_owned(),
            channel,
            metadata: String::new(),
            is_talking: false,
            input_muted: 0,
            output_muted: 0,
            input_deactivated: 0,
            is_muted: false,
        }
    }
}

impl FakeServer {
    /// The next free client ID on this server.
    pub(crate) fn next_client_id(&self) -> Option<ClientId> {
        let highest = self.clients.keys().next_back().map_or(0, |client| client.0);
        highest.checked_add(1).map(ClientId)
    }
}

impl FakeTeamSpeak {
    /// A client with no server, no setting and no record.
    pub(crate) const fn empty() -> Self {
        Self {
            servers: BTreeMap::new(),
            current_server: ServerConnectionHandlerId(0),
            preprocessor_values: BTreeMap::new(),
            plugin_path: PathBuf::new(),
            config_path: PathBuf::new(),
            calls: Vec::new(),
            ledger: AllocationLedger::empty(),
        }
    }

    /// A fresh client answering with `plugin_path` and `config_path`, voice activation off.
    pub(crate) fn fresh(plugin_path: PathBuf, config_path: PathBuf) -> Self {
        let mut fake = Self::empty();
        fake.plugin_path = plugin_path;
        fake.config_path = config_path;
        fake.preprocessor_values
            .insert("vad".to_owned(), "false".to_owned());
        fake
    }

    /// Appends `call` to the log.
    pub(crate) fn record(&mut self, call: HostCall) {
        self.calls.push(call);
    }

    /// The calls logged so far, in order.
    pub fn calls(&self) -> &[HostCall] {
        &self.calls
    }

    /// The server tab `server`, if it exists.
    pub fn server(&self, server: ServerConnectionHandlerId) -> Option<&FakeServer> {
        self.servers.get(&server)
    }

    /// The server tab `server` for changing, if it exists.
    pub fn server_mut(&mut self, server: ServerConnectionHandlerId) -> Option<&mut FakeServer> {
        self.servers.get_mut(&server)
    }

    /// The client `client` on `server`, if both exist.
    pub fn client(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<&FakeClient> {
        self.server(server)?.clients.get(&client)
    }

    /// The client `client` on `server` for changing, if both exist.
    pub fn client_mut(
        &mut self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<&mut FakeClient> {
        self.server_mut(server)?.clients.get_mut(&client)
    }

    /// Our own client on `server`, if the server exists.
    pub fn own_client_mut(&mut self, server: ServerConnectionHandlerId) -> Option<&mut FakeClient> {
        let fake_server = self.server_mut(server)?;
        let own = fake_server.own_client;
        fake_server.clients.get_mut(&own)
    }
}
