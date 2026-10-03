//! [`TeamSpeakActions`]: every TeamSpeak query and side effect the plugin makes, behind one
//! trait, and its implementation over the installed function table.
//!
//! **Role:** the plugin's only door to TeamSpeak. The runtime asks it for server, channel and
//! client facts and tells it to mute, rename, publish metadata, send plugin commands, play
//! sounds, move the own client, neutralise TeamSpeak's 3D sound and switch voice activation and
//! input; a failed TeamSpeak call is logged here and reported as `false` or `None`.
//!
//! **Position:** implemented for `tfar_ts3_host::Ts3`; held by `runtime::PluginRuntime` as a
//! `&'static dyn TeamSpeakActions`, so unit tests can hand in a recording fake.
//!
//! **Signals and state:** none; every method may run on any plugin thread (the game link
//! threads, the service thread, the tangent release thread, TeamSpeak's main thread), never on
//! an audio thread.
//!
//! **Invariants:** no caller holds a server lock while calling a method here, as TeamSpeak may
//! call back into the plugin on the calling thread; text arguments go to TeamSpeak unchanged.

use std::path::{Path, PathBuf};

use tfar_ts3_host::{
    ChannelId, ChannelProperty, ClientId, ClientProperty, ConnectStatus, PluginCommandTarget,
    ServerConnectionHandlerId, ServerProperty, Ts3, Ts3Error, Ts3Vector,
};

/// The pre-processor setting that switches voice activation (`"vad"`, `"true"` or `"false"`).
pub const VOICE_ACTIVATION_SETTING: &str = "vad";

/// `CLIENT_INPUT_DEACTIVATED` while the microphone sends (`INPUT_ACTIVE`).
pub const INPUT_ACTIVE: i32 = 0;

/// `CLIENT_INPUT_DEACTIVATED` while the microphone is off (`INPUT_DEACTIVATED`).
pub const INPUT_DEACTIVATED: i32 = 1;

/// The TeamSpeak queries and side effects of the plugin; see the module header.
pub trait TeamSpeakActions: Send + Sync {
    /// The server tab in front.
    fn current_server(&self) -> ServerConnectionHandlerId;
    /// Whether `server` has finished connecting.
    fn is_connected(&self, server: ServerConnectionHandlerId) -> bool;
    /// The local client's id on `server`.
    fn own_client_id(&self, server: ServerConnectionHandlerId) -> Option<ClientId>;
    /// `client`'s nickname.
    fn client_nickname(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<String>;
    /// `client`'s metadata (`CLIENT_META_DATA`).
    fn client_metadata(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<String>;
    /// The channel `client` is in.
    fn channel_of_client(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<ChannelId>;
    /// The clients in `channel`.
    fn channel_clients(
        &self,
        server: ServerConnectionHandlerId,
        channel: ChannelId,
    ) -> Vec<ClientId>;
    /// Every channel of `server` with its name, in TeamSpeak's order.
    fn channels(&self, server: ServerConnectionHandlerId) -> Vec<(ChannelId, String)>;
    /// The server's name.
    fn server_name(&self, server: ServerConnectionHandlerId) -> Option<String>;
    /// The server's unique identifier.
    fn server_unique_identifier(&self, server: ServerConnectionHandlerId) -> Option<String>;
    /// `channel`'s name.
    fn channel_name(&self, server: ServerConnectionHandlerId, channel: ChannelId)
    -> Option<String>;
    /// TeamSpeak's plugins folder.
    fn plugin_path(&self) -> Option<PathBuf>;
    /// TeamSpeak's configuration folder.
    fn config_path(&self) -> Option<PathBuf>;
    /// Whether voice activation is on (`vad` is `true`).
    fn is_voice_activation_enabled(&self, server: ServerConnectionHandlerId) -> bool;

    /// Mutes `clients` for the local player.
    fn mute_clients(&self, server: ServerConnectionHandlerId, clients: &[ClientId]);
    /// Unmutes `clients` for the local player.
    fn unmute_clients(&self, server: ServerConnectionHandlerId, clients: &[ClientId]);
    /// Renames the local client; whether TeamSpeak took the name.
    fn set_own_nickname(&self, server: ServerConnectionHandlerId, nickname: &str) -> bool;
    /// Replaces the local client's metadata.
    fn set_own_metadata(&self, server: ServerConnectionHandlerId, metadata: &str);
    /// Sends a plugin command.
    fn send_plugin_command(
        &self,
        server: ServerConnectionHandlerId,
        command: &str,
        target: PluginCommandTarget<'_>,
    );
    /// Plays a WAV file through TeamSpeak's own playback.
    fn play_wave_file(&self, server: ServerConnectionHandlerId, path: &Path);
    /// Asks to move the local client into `channel` with `password`.
    fn move_own_client(
        &self,
        server: ServerConnectionHandlerId,
        channel: ChannelId,
        password: &str,
    );
    /// TeamSpeak's own 3D attenuation off (`systemset3DSettings(1, 1)`).
    fn neutralise_3d_settings(&self, server: ServerConnectionHandlerId);
    /// The 3D listener at the origin, its forward and up vectors left as TeamSpeak has them
    /// (`systemset3DListenerAttributes(pos, nullptr, nullptr)`, `Teamspeak.cpp:686`).
    fn neutralise_listener(&self, server: ServerConnectionHandlerId);
    /// `client`'s 3D voice position at the origin.
    fn neutralise_client(&self, server: ServerConnectionHandlerId, client: ClientId);
    /// Switches voice activation on or off.
    fn set_voice_activation(&self, server: ServerConnectionHandlerId, is_enabled: bool);
    /// Switches the microphone off (`true`) or on, and flushes the change.
    fn set_input_deactivated(&self, server: ServerConnectionHandlerId, is_deactivated: bool);
    /// Prints a line in the current chat tab.
    fn print_to_current_tab(&self, text: &str);
}

/// Logs a failed TeamSpeak call and turns the result into an `Option`.
fn logged<T>(what: &'static str, result: Result<T, Ts3Error>) -> Option<T> {
    result
        .inspect_err(|error| tracing::warn!(%error, "TeamSpeak call failed: {what}"))
        .ok()
}

/// The origin, where TeamSpeak's own 3D places every voice so it adds no panning.
const ORIGIN: Ts3Vector = Ts3Vector {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

impl TeamSpeakActions for Ts3 {
    fn current_server(&self) -> ServerConnectionHandlerId {
        self.current_server_connection_handler_id()
    }

    fn is_connected(&self, server: ServerConnectionHandlerId) -> bool {
        matches!(
            self.connection_status(server),
            Ok(ConnectStatus::ConnectionEstablished)
        )
    }

    fn own_client_id(&self, server: ServerConnectionHandlerId) -> Option<ClientId> {
        logged("getClientID", self.client_id(server))
    }

    fn client_nickname(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<String> {
        logged(
            "getClientVariableAsString(NICKNAME)",
            self.client_variable_as_string(server, client, ClientProperty::Nickname),
        )
    }

    fn client_metadata(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<String> {
        logged(
            "getClientVariableAsString(META_DATA)",
            self.client_variable_as_string(server, client, ClientProperty::MetaData),
        )
    }

    fn channel_of_client(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<ChannelId> {
        logged(
            "getChannelOfClient",
            Ts3::channel_of_client(self, server, client),
        )
    }

    fn channel_clients(
        &self,
        server: ServerConnectionHandlerId,
        channel: ChannelId,
    ) -> Vec<ClientId> {
        logged(
            "getChannelClientList",
            self.channel_client_list(server, channel),
        )
        .unwrap_or_default()
    }

    fn channels(&self, server: ServerConnectionHandlerId) -> Vec<(ChannelId, String)> {
        logged("getChannelList", self.channel_list(server))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|channel| {
                TeamSpeakActions::channel_name(self, server, channel).map(|name| (channel, name))
            })
            .collect()
    }

    fn server_name(&self, server: ServerConnectionHandlerId) -> Option<String> {
        logged(
            "getServerVariableAsString(NAME)",
            self.server_variable_as_string(server, ServerProperty::Name),
        )
    }

    fn server_unique_identifier(&self, server: ServerConnectionHandlerId) -> Option<String> {
        logged(
            "getServerVariableAsString(UNIQUE_IDENTIFIER)",
            self.server_variable_as_string(server, ServerProperty::UniqueIdentifier),
        )
    }

    fn channel_name(
        &self,
        server: ServerConnectionHandlerId,
        channel: ChannelId,
    ) -> Option<String> {
        logged(
            "getChannelVariableAsString(NAME)",
            self.channel_variable_as_string(server, channel, ChannelProperty::Name),
        )
    }

    fn plugin_path(&self) -> Option<PathBuf> {
        logged("getPluginPath", Ts3::plugin_path(self))
    }

    fn config_path(&self) -> Option<PathBuf> {
        logged("getConfigPath", Ts3::config_path(self))
    }

    fn is_voice_activation_enabled(&self, server: ServerConnectionHandlerId) -> bool {
        logged(
            "getPreProcessorConfigValue(vad)",
            self.pre_processor_config_value(server, VOICE_ACTIVATION_SETTING),
        )
        .is_some_and(|value| value == "true")
    }

    fn mute_clients(&self, server: ServerConnectionHandlerId, clients: &[ClientId]) {
        if !clients.is_empty() {
            logged(
                "requestMuteClients",
                self.request_mute_clients(server, clients),
            );
        }
    }

    fn unmute_clients(&self, server: ServerConnectionHandlerId, clients: &[ClientId]) {
        if !clients.is_empty() {
            logged(
                "requestUnmuteClients",
                self.request_unmute_clients(server, clients),
            );
        }
    }

    fn set_own_nickname(&self, server: ServerConnectionHandlerId, nickname: &str) -> bool {
        let is_set = logged(
            "setClientSelfVariableAsString(NICKNAME)",
            self.set_client_self_variable_as_string(server, ClientProperty::Nickname, nickname),
        )
        .is_some();
        logged(
            "flushClientSelfUpdates",
            self.flush_client_self_updates(server),
        );
        is_set
    }

    fn set_own_metadata(&self, server: ServerConnectionHandlerId, metadata: &str) {
        logged(
            "setClientSelfVariableAsString(META_DATA)",
            self.set_client_self_variable_as_string(server, ClientProperty::MetaData, metadata),
        );
        logged(
            "flushClientSelfUpdates",
            self.flush_client_self_updates(server),
        );
    }

    fn send_plugin_command(
        &self,
        server: ServerConnectionHandlerId,
        command: &str,
        target: PluginCommandTarget<'_>,
    ) {
        logged(
            "sendPluginCommand",
            Ts3::send_plugin_command(self, server, command, target),
        );
    }

    fn play_wave_file(&self, server: ServerConnectionHandlerId, path: &Path) {
        logged("playWaveFile", Ts3::play_wave_file(self, server, path));
    }

    fn move_own_client(
        &self,
        server: ServerConnectionHandlerId,
        channel: ChannelId,
        password: &str,
    ) {
        if let Some(own) = TeamSpeakActions::own_client_id(self, server) {
            logged(
                "requestClientMove",
                self.request_client_move(server, own, channel, password),
            );
        }
    }

    fn neutralise_3d_settings(&self, server: ServerConnectionHandlerId) {
        logged(
            "systemset3DSettings",
            self.system_set_3d_settings(server, 1.0, 1.0),
        );
    }

    fn neutralise_listener(&self, server: ServerConnectionHandlerId) {
        logged(
            "systemset3DListenerAttributes",
            self.system_set_3d_listener_attributes(server, ORIGIN, None, None),
        );
    }

    fn neutralise_client(&self, server: ServerConnectionHandlerId, client: ClientId) {
        logged(
            "channelset3DAttributes",
            self.channel_set_3d_attributes(server, client, ORIGIN),
        );
    }

    fn set_voice_activation(&self, server: ServerConnectionHandlerId, is_enabled: bool) {
        let value = if is_enabled { "true" } else { "false" };
        logged(
            "setPreProcessorConfigValue(vad)",
            self.set_pre_processor_config_value(server, VOICE_ACTIVATION_SETTING, value),
        );
    }

    fn set_input_deactivated(&self, server: ServerConnectionHandlerId, is_deactivated: bool) {
        let value = if is_deactivated {
            INPUT_DEACTIVATED
        } else {
            INPUT_ACTIVE
        };
        logged(
            "setClientSelfVariableAsInt(INPUT_DEACTIVATED)",
            self.set_client_self_variable_as_int(server, ClientProperty::InputDeactivated, value),
        );
        logged(
            "flushClientSelfUpdates",
            self.flush_client_self_updates(server),
        );
    }

    fn print_to_current_tab(&self, text: &str) {
        logged(
            "printMessageToCurrentTab",
            self.print_message_to_current_tab(text),
        );
    }
}
