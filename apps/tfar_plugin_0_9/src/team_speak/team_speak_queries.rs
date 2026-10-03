//! `TeamSpeakQueries`: everything the plugin reads from TeamSpeak, as one trait.
//!
//! **Role:** name each TeamSpeak read the 0.9 plugin makes (`getClientID`, `getChannelOfClient`,
//! `getChannelVariableAsString`, the client lists, the nickname, the metadata, the talking flag,
//! the server name, the `vad` pre-processor value), with a failed read folded into `None`, an
//! empty list or `false` the way the original treats its failures.
//!
//! **Position:** a supertrait of [`crate::team_speak::TeamSpeak`]; implemented over the function
//! table by `Ts3TeamSpeak` and by the recording fake of the unit tests.
//!
//! **Signals and state:** none of its own. Called on TeamSpeak's main thread, the game link
//! thread and the service thread, never on an audio thread and never under a server lock.
//!
//! **Invariants:** every method answers; a TeamSpeak error is logged by the implementation and
//! becomes the neutral answer, never a panic.

use tfar_ts3_host::{ChannelId, ClientId, ServerConnectionHandlerId};

/// The TeamSpeak reads of the 0.9 plugin.
pub trait TeamSpeakQueries {
    /// The server tab in front (`getCurrentServerConnectionHandlerID`); `0` without one.
    fn current_server(&self) -> ServerConnectionHandlerId;

    /// Every server tab whose connection is established, so channels and clients are known.
    fn connected_servers(&self) -> Vec<ServerConnectionHandlerId>;

    /// Whether the connection of `server` is established (`isConnected`).
    fn is_connected(&self, server: ServerConnectionHandlerId) -> bool;

    /// Our own client id on `server` (`getClientID`).
    fn my_client_id(&self, server: ServerConnectionHandlerId) -> Option<ClientId>;

    /// The channel `client` sits in (`getChannelOfClient`).
    fn channel_of_client(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<ChannelId>;

    /// The name of `channel` (`getChannelVariableAsString(CHANNEL_NAME)`).
    fn channel_name(&self, server: ServerConnectionHandlerId, channel: ChannelId)
    -> Option<String>;

    /// Every channel on `server` (`getChannelList`), in TeamSpeak's order.
    fn channel_list(&self, server: ServerConnectionHandlerId) -> Vec<ChannelId>;

    /// The clients in `channel` (`getChannelClientList`).
    fn channel_clients(
        &self,
        server: ServerConnectionHandlerId,
        channel: ChannelId,
    ) -> Vec<ClientId>;

    /// The nickname of `client` (`getClientVariableAsString(CLIENT_NICKNAME)`).
    fn client_nickname(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<String>;

    /// The whole metadata of `client` (`getClientVariableAsString(CLIENT_META_DATA)`).
    fn client_metadata(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<String>;

    /// Whether TeamSpeak's talking flag of `client` is set (`isTalking`: our own flag through
    /// `getClientSelfVariableAsInt`, anyone else's through `getClientVariableAsInt`).
    fn is_client_talking(&self, server: ServerConnectionHandlerId, client: ClientId) -> bool;

    /// The virtual server's name (`getServerVariableAsString(VIRTUALSERVER_NAME)`).
    fn server_name(&self, server: ServerConnectionHandlerId) -> Option<String>;

    /// Whether voice activation is on: the `vad` pre-processor value is exactly `true`
    /// (`hlp_checkVad`).
    fn is_voice_activation_on(&self, server: ServerConnectionHandlerId) -> bool;
}
