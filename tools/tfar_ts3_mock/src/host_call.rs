//! [`HostCall`]: one entry of the mock's call log, a typed record of a TeamSpeak function the
//! plugin called.
//!
//! **Role:** the vocabulary tests assert in: every fake in `fake_functions` appends one value
//! before it answers, with its arguments copied out of the plugin's memory.
//!
//! **Position:** written by `fake_functions`, stored in the fake state, read through
//! [`MockHost::calls`](crate::MockHost::calls) and printed by the binary.
//!
//! **Signals and state:** plain values; no state.
//!
//! **Invariants:** a record owns all its data (no pointer into plugin memory survives the call);
//! `freeMemory` is not logged, since the allocation ledger accounts for it.

use std::path::PathBuf;

use tfar_ts3_host::{ChannelId, ClientId, LogLevel, ServerConnectionHandlerId, Ts3Vector};

/// One TeamSpeak function call the plugin made, with its arguments.
#[derive(Debug, Clone, PartialEq)]
pub enum HostCall {
    /// A read-only query; its answer comes from the fake state.
    Query(HostQuery),
    /// `logMessage`.
    LogMessage {
        /// The log line.
        text: String,
        /// Its severity.
        level: LogLevel,
        /// The log channel (the plugin's name in TFAR).
        channel: String,
        /// The server the line belongs to.
        server: ServerConnectionHandlerId,
    },
    /// `printMessageToCurrentTab`.
    PrintMessageToCurrentTab {
        /// The BB-code text.
        text: String,
    },
    /// `sendPluginCommand`.
    SendPluginCommand {
        /// The server the command goes through.
        server: ServerConnectionHandlerId,
        /// The plugin ID the plugin passed.
        plugin_id: String,
        /// The command text.
        command: String,
        /// Who receives it.
        target: SentCommandTarget,
    },
    /// `requestMuteClients`.
    RequestMuteClients {
        /// The server.
        server: ServerConnectionHandlerId,
        /// The clients muted, terminator dropped.
        clients: Vec<ClientId>,
    },
    /// `requestUnmuteClients`.
    RequestUnmuteClients {
        /// The server.
        server: ServerConnectionHandlerId,
        /// The clients unmuted, terminator dropped.
        clients: Vec<ClientId>,
    },
    /// `requestClientMove`; the mock records it and moves nobody (a test delivers the move).
    RequestClientMove {
        /// The server.
        server: ServerConnectionHandlerId,
        /// The client to move.
        client: ClientId,
        /// The channel asked for.
        channel: ChannelId,
        /// The channel password, empty for none.
        password: String,
    },
    /// `setClientSelfVariableAsString`.
    SetClientSelfVariableAsString {
        /// The server.
        server: ServerConnectionHandlerId,
        /// The SDK `ClientProperties` flag.
        flag: usize,
        /// The new value.
        value: String,
    },
    /// `setClientSelfVariableAsInt`.
    SetClientSelfVariableAsInt {
        /// The server.
        server: ServerConnectionHandlerId,
        /// The SDK `ClientProperties` flag.
        flag: usize,
        /// The new value.
        value: i32,
    },
    /// `flushClientSelfUpdates`.
    FlushClientSelfUpdates {
        /// The server.
        server: ServerConnectionHandlerId,
    },
    /// `playWaveFile`.
    PlayWaveFile {
        /// The server whose playback device plays it.
        server: ServerConnectionHandlerId,
        /// The WAV file.
        path: PathBuf,
    },
    /// `systemset3DSettings`.
    Systemset3DSettings {
        /// The server.
        server: ServerConnectionHandlerId,
        /// The distance factor.
        distance_factor: f32,
        /// The rolloff scale.
        rolloff_scale: f32,
    },
    /// `systemset3DListenerAttributes`; a null vector is `None`.
    Systemset3DListenerAttributes {
        /// The server.
        server: ServerConnectionHandlerId,
        /// The listener position.
        position: Option<Ts3Vector>,
        /// The forward direction.
        forward: Option<Ts3Vector>,
        /// The up direction.
        up: Option<Ts3Vector>,
    },
    /// `channelset3DAttributes`; a null vector is `None`.
    Channelset3DAttributes {
        /// The server.
        server: ServerConnectionHandlerId,
        /// The client whose voice is placed.
        client: ClientId,
        /// Its position.
        position: Option<Ts3Vector>,
    },
    /// `setPreProcessorConfigValue`.
    SetPreProcessorConfigValue {
        /// The server.
        server: ServerConnectionHandlerId,
        /// The setting, such as `vad`.
        ident: String,
        /// The new value.
        value: String,
    },
    /// A table slot no fake implements; it answers `ERROR_not_implemented`.
    UnexpectedFunction {
        /// The slot's index in `TS3Functions`, in SDK order.
        slot: usize,
    },
}

/// A read-only TeamSpeak query the plugin made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostQuery {
    /// `getCurrentServerConnectionHandlerID`.
    CurrentServerConnectionHandlerId,
    /// `getServerConnectionHandlerList`.
    ServerConnectionHandlerList,
    /// `getConnectionStatus`.
    ConnectionStatus(ServerConnectionHandlerId),
    /// `getClientID`.
    ClientId(ServerConnectionHandlerId),
    /// `getClientList`.
    ClientList(ServerConnectionHandlerId),
    /// `getChannelList`.
    ChannelList(ServerConnectionHandlerId),
    /// `getChannelClientList`.
    ChannelClientList(ServerConnectionHandlerId, ChannelId),
    /// `getChannelOfClient`.
    ChannelOfClient(ServerConnectionHandlerId, ClientId),
    /// `getClientVariableAsString` with the SDK flag.
    ClientVariableAsString(ServerConnectionHandlerId, ClientId, usize),
    /// `getClientVariableAsInt` with the SDK flag.
    ClientVariableAsInt(ServerConnectionHandlerId, ClientId, usize),
    /// `getClientSelfVariableAsInt` with the SDK flag.
    ClientSelfVariableAsInt(ServerConnectionHandlerId, usize),
    /// `getChannelVariableAsString` with the SDK flag.
    ChannelVariableAsString(ServerConnectionHandlerId, ChannelId, usize),
    /// `getServerVariableAsString` with the SDK flag.
    ServerVariableAsString(ServerConnectionHandlerId, usize),
    /// `getErrorMessage` for the code.
    ErrorMessage(u32),
    /// `getPreProcessorConfigValue` for the setting named.
    PreProcessorConfigValue(ServerConnectionHandlerId, String),
    /// `getPluginPath`.
    PluginPath,
    /// `getConfigPath`.
    ConfigPath,
}

/// The receivers of a `sendPluginCommand`, as the plugin named them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SentCommandTarget {
    /// `PluginCommandTarget_CURRENT_CHANNEL`.
    CurrentChannel,
    /// `PluginCommandTarget_SERVER`.
    Server,
    /// `PluginCommandTarget_CLIENT` with the listed clients, terminator dropped.
    Clients(Vec<ClientId>),
    /// `PluginCommandTarget_CURRENT_CHANNEL_SUBSCRIBED_CLIENTS`.
    CurrentChannelSubscribedClients,
    /// A mode the SDK does not define.
    Other(i32),
}
