//! `Ts3TeamSpeak`: [`TeamSpeakQueries`] and [`TeamSpeakActions`] over TeamSpeak's function table.
//!
//! **Role:** the production implementation: each method is one or two calls on the safe
//! [`Ts3`] handle of `tfar_ts3_host`, with the failure logged and turned into the neutral answer
//! the original falls back to.
//!
//! **Position:** made once in `ts3plugin_init` and shared as the plugin's `Arc<dyn TeamSpeak>`.
//!
//! **Signals and state:** the `'static` function table only. Its methods run on the plugin's
//! command threads (TeamSpeak's main thread, the game link thread, the service and push-to-talk
//! threads), never on an audio thread.
//!
//! **Invariants:** a read never fails loudly: an error is a debug log line and `None`, an empty
//! list or `false`; a write returns TeamSpeak's error to the caller.

use std::path::Path;

use tfar_ts3_host::{
    ChannelId, ChannelProperty, ClientId, ClientProperty, ConnectStatus, PluginCommandTarget,
    ServerConnectionHandlerId, ServerProperty, Ts3, Ts3Error, Ts3Vector,
};

use super::team_speak_actions::TeamSpeakActions;
use super::team_speak_queries::TeamSpeakQueries;

/// The `CLIENT_INPUT_DEACTIVATED` value that captures from the microphone (`INPUT_ACTIVE`).
const INPUT_ACTIVE: i32 = 0;

/// The `CLIENT_INPUT_DEACTIVATED` value that captures nothing (`INPUT_DEACTIVATED`).
const INPUT_DEACTIVATED: i32 = 1;

/// The pre-processor setting of voice activation.
const VOICE_ACTIVATION_SETTING: &str = "vad";

/// TeamSpeak's own function table, as the plugin's [`TeamSpeakQueries`] and
/// [`TeamSpeakActions`].
#[derive(Clone, Copy)]
pub struct Ts3TeamSpeak {
    ts3: &'static Ts3,
}

impl core::fmt::Debug for Ts3TeamSpeak {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Ts3TeamSpeak")
            .finish_non_exhaustive()
    }
}

impl Ts3TeamSpeak {
    /// The plugin's TeamSpeak over the installed function table `ts3`.
    pub const fn new(ts3: &'static Ts3) -> Self {
        Self { ts3 }
    }
}

/// `result`'s value, or `None` with a debug line naming `what`.
fn logged<Value>(what: &'static str, result: Result<Value, Ts3Error>) -> Option<Value> {
    result
        .map_err(|error| tracing::debug!(%error, "TeamSpeak could not tell {what}"))
        .ok()
}

impl TeamSpeakQueries for Ts3TeamSpeak {
    fn current_server(&self) -> ServerConnectionHandlerId {
        self.ts3.current_server_connection_handler_id()
    }

    fn connected_servers(&self) -> Vec<ServerConnectionHandlerId> {
        logged("the server tabs", self.ts3.server_connection_handler_list())
            .unwrap_or_default()
            .into_iter()
            .filter(|server| self.is_connected(*server))
            .collect()
    }

    fn is_connected(&self, server: ServerConnectionHandlerId) -> bool {
        server.0 != 0
            && self.ts3.connection_status(server).ok() == Some(ConnectStatus::ConnectionEstablished)
    }

    fn my_client_id(&self, server: ServerConnectionHandlerId) -> Option<ClientId> {
        if !self.is_connected(server) {
            return None;
        }
        logged("our client id", self.ts3.client_id(server))
    }

    fn channel_of_client(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<ChannelId> {
        logged(
            "a client's channel",
            self.ts3.channel_of_client(server, client),
        )
    }

    fn channel_name(
        &self,
        server: ServerConnectionHandlerId,
        channel: ChannelId,
    ) -> Option<String> {
        logged(
            "a channel name",
            self.ts3
                .channel_variable_as_string(server, channel, ChannelProperty::Name),
        )
    }

    fn channel_list(&self, server: ServerConnectionHandlerId) -> Vec<ChannelId> {
        logged("the channel list", self.ts3.channel_list(server)).unwrap_or_default()
    }

    fn channel_clients(
        &self,
        server: ServerConnectionHandlerId,
        channel: ChannelId,
    ) -> Vec<ClientId> {
        logged(
            "the channel client list",
            self.ts3.channel_client_list(server, channel),
        )
        .unwrap_or_default()
    }

    fn client_nickname(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<String> {
        logged(
            "a nickname",
            self.ts3
                .client_variable_as_string(server, client, ClientProperty::Nickname),
        )
    }

    fn client_metadata(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<String> {
        logged(
            "a client's metadata",
            self.ts3
                .client_variable_as_string(server, client, ClientProperty::MetaData),
        )
    }

    fn is_client_talking(&self, server: ServerConnectionHandlerId, client: ClientId) -> bool {
        let flag = if self.my_client_id(server) == Some(client) {
            self.ts3
                .client_self_variable_as_int(server, ClientProperty::FlagTalking)
        } else {
            self.ts3
                .client_variable_as_int(server, client, ClientProperty::FlagTalking)
        };
        logged("a talking flag", flag) == Some(1)
    }

    fn server_name(&self, server: ServerConnectionHandlerId) -> Option<String> {
        logged(
            "the server name",
            self.ts3
                .server_variable_as_string(server, ServerProperty::Name),
        )
    }

    fn is_voice_activation_on(&self, server: ServerConnectionHandlerId) -> bool {
        logged(
            "the voice activation setting",
            self.ts3
                .pre_processor_config_value(server, VOICE_ACTIVATION_SETTING),
        )
        .is_some_and(|value| value == "true")
    }
}

impl TeamSpeakActions for Ts3TeamSpeak {
    fn mute_clients(
        &self,
        server: ServerConnectionHandlerId,
        clients: &[ClientId],
    ) -> Result<(), Ts3Error> {
        let result = self.ts3.request_mute_clients(server, clients);
        tracing::info!(clients = ?clients, ok = result.is_ok(), "mute: requested");
        result
    }

    fn unmute_clients(
        &self,
        server: ServerConnectionHandlerId,
        clients: &[ClientId],
    ) -> Result<(), Ts3Error> {
        let result = self.ts3.request_unmute_clients(server, clients);
        tracing::info!(clients = ?clients, ok = result.is_ok(), "mute: unmute requested");
        result
    }

    fn set_own_nickname(
        &self,
        server: ServerConnectionHandlerId,
        nickname: &str,
    ) -> Result<(), Ts3Error> {
        self.ts3
            .set_client_self_variable_as_string(server, ClientProperty::Nickname, nickname)?;
        self.ts3.flush_client_self_updates(server)
    }

    fn set_own_metadata(
        &self,
        server: ServerConnectionHandlerId,
        metadata: &str,
    ) -> Result<(), Ts3Error> {
        self.ts3
            .set_client_self_variable_as_string(server, ClientProperty::MetaData, metadata)?;
        self.ts3.flush_client_self_updates(server)
    }

    fn send_plugin_command(
        &self,
        server: ServerConnectionHandlerId,
        command: &str,
    ) -> Result<(), Ts3Error> {
        self.ts3
            .send_plugin_command(server, command, PluginCommandTarget::CurrentChannel)
    }

    fn play_wave_file(
        &self,
        server: ServerConnectionHandlerId,
        path: &Path,
    ) -> Result<(), Ts3Error> {
        self.ts3.play_wave_file(server, path)
    }

    fn move_client(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
        channel: ChannelId,
        password: &str,
    ) -> Result<(), Ts3Error> {
        self.ts3
            .request_client_move(server, client, channel, password)
    }

    fn neutralise_3d_settings(&self, server: ServerConnectionHandlerId) -> Result<(), Ts3Error> {
        self.ts3.system_set_3d_settings(server, 1.0, 1.0)
    }

    fn centre_listener(&self, server: ServerConnectionHandlerId) -> Result<(), Ts3Error> {
        // No orientation (NULL forward and up), as the original passes; TeamSpeak keeps its own.
        self.ts3
            .system_set_3d_listener_attributes(server, Ts3Vector::default(), None, None)
    }

    fn centre_client(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Result<(), Ts3Error> {
        self.ts3
            .channel_set_3d_attributes(server, client, Ts3Vector::default())
    }

    fn set_voice_activation(
        &self,
        server: ServerConnectionHandlerId,
        is_on: bool,
    ) -> Result<(), Ts3Error> {
        let value = if is_on { "true" } else { "false" };
        self.ts3
            .set_pre_processor_config_value(server, VOICE_ACTIVATION_SETTING, value)
    }

    fn set_input_active(
        &self,
        server: ServerConnectionHandlerId,
        is_active: bool,
    ) -> Result<(), Ts3Error> {
        let value = if is_active {
            INPUT_ACTIVE
        } else {
            INPUT_DEACTIVATED
        };
        self.ts3.set_client_self_variable_as_int(
            server,
            ClientProperty::InputDeactivated,
            value,
        )?;
        self.ts3.flush_client_self_updates(server)
    }

    fn print_message(&self, text: &str) {
        if let Err(error) = self.ts3.print_message_to_current_tab(text) {
            tracing::debug!(%error, "a chat line could not be printed");
        }
    }
}
