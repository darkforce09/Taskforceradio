//! The SDK values `tfar_ts3_host` passes to TeamSpeak or reads from it: connection status,
//! visibility, talk status, log level, property flags, plugin command targets and 3D vectors.
//!
//! **Role:** typed stand-ins for the C enums, so no caller passes a bare integer flag, and so no
//! integer from TeamSpeak is ever transmuted into a Rust enum (an unknown value stays representable).
//!
//! **Position:** shared by `ts3_functions` (arguments) and `plugin_exports` (callback arguments).
//!
//! **Signals and state:** plain values; no state.
//!
//! **Invariants:** every discriminant equals the SDK header value; `src/tests/sdk_layout.rs` parses
//! the headers under `sdk_headers/` and checks each one.

use ts3plugin_sys::public_definitions as sdk;

/// `enum ConnectStatus`: how far a server connection has come.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectStatus {
    /// `STATUS_DISCONNECTED`.
    Disconnected,
    /// `STATUS_CONNECTING`.
    Connecting,
    /// `STATUS_CONNECTED`: accepted, but channels and clients are not known yet.
    Connected,
    /// `STATUS_CONNECTION_ESTABLISHING`.
    ConnectionEstablishing,
    /// `STATUS_CONNECTION_ESTABLISHED`: channels and clients are available.
    ConnectionEstablished,
    /// A value the SDK of plugin API 26 does not define.
    Other(i32),
}

impl ConnectStatus {
    /// The status for TeamSpeak's integer value.
    pub fn from_raw(value: i32) -> Self {
        match value {
            0 => Self::Disconnected,
            1 => Self::Connecting,
            2 => Self::Connected,
            3 => Self::ConnectionEstablishing,
            4 => Self::ConnectionEstablished,
            other => Self::Other(other),
        }
    }

    /// TeamSpeak's integer value of this status.
    pub fn to_raw(self) -> i32 {
        match self {
            Self::Disconnected => sdk::ConnectStatus::Disconnected as i32,
            Self::Connecting => sdk::ConnectStatus::Connecting as i32,
            Self::Connected => sdk::ConnectStatus::Connected as i32,
            Self::ConnectionEstablishing => sdk::ConnectStatus::ConnectionEstablishing as i32,
            Self::ConnectionEstablished => sdk::ConnectStatus::ConnectionEstablished as i32,
            Self::Other(value) => value,
        }
    }
}

/// `enum Visibility`: whether a moved client enters, stays in or leaves our view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// `ENTER_VISIBILITY`.
    Enter,
    /// `RETAIN_VISIBILITY`.
    Retain,
    /// `LEAVE_VISIBILITY`.
    Leave,
    /// A value the SDK of plugin API 26 does not define.
    Other(i32),
}

impl Visibility {
    /// The visibility for TeamSpeak's integer value.
    pub fn from_raw(value: i32) -> Self {
        match value {
            0 => Self::Enter,
            1 => Self::Retain,
            2 => Self::Leave,
            other => Self::Other(other),
        }
    }
}

/// `enum TalkStatus`: whether a client is talking, as `onTalkStatusChangeEvent` reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TalkStatus {
    /// `STATUS_NOT_TALKING`.
    NotTalking,
    /// `STATUS_TALKING`.
    Talking,
    /// `STATUS_TALKING_WHILE_DISABLED`: talking with the microphone muted (own client only).
    TalkingWhileDisabled,
    /// A value the SDK of plugin API 26 does not define.
    Other(i32),
}

impl TalkStatus {
    /// The status for TeamSpeak's integer value.
    pub fn from_raw(value: i32) -> Self {
        match value {
            0 => Self::NotTalking,
            1 => Self::Talking,
            2 => Self::TalkingWhileDisabled,
            other => Self::Other(other),
        }
    }

    /// TeamSpeak's integer value of this status.
    pub fn to_raw(self) -> i32 {
        match self {
            Self::NotTalking => sdk::TalkStatus::NotTalking as i32,
            Self::Talking => sdk::TalkStatus::Talking as i32,
            Self::TalkingWhileDisabled => sdk::TalkStatus::TalkingWhileDisabled as i32,
            Self::Other(value) => value,
        }
    }

    /// True for [`Talking`](Self::Talking) only: a client talking while its microphone is
    /// disabled sends no voice.
    pub fn is_talking(self) -> bool {
        matches!(self, Self::Talking)
    }
}

/// `enum LogLevel` of `teamlog/logtypes.h`: the severity of a `logMessage` line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// `LogLevel_CRITICAL`: the program stops.
    Critical,
    /// `LogLevel_ERROR`.
    Error,
    /// `LogLevel_WARNING`.
    Warning,
    /// `LogLevel_DEBUG`.
    Debug,
    /// `LogLevel_INFO`.
    Info,
    /// `LogLevel_DEVEL`: developer output, hidden in release clients.
    Devel,
}

impl LogLevel {
    /// The `ts3plugin-sys` value `logMessage` takes.
    pub(crate) fn to_sdk(self) -> sdk::LogLevel {
        match self {
            Self::Critical => sdk::LogLevel::Critical,
            Self::Error => sdk::LogLevel::Error,
            Self::Warning => sdk::LogLevel::Warning,
            Self::Debug => sdk::LogLevel::Debug,
            Self::Info => sdk::LogLevel::Info,
            Self::Devel => sdk::LogLevel::Devel,
        }
    }
}

/// The `enum ClientProperties` flags TFAR reads or writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientProperty {
    /// `CLIENT_UNIQUE_IDENTIFIER` (string).
    UniqueIdentifier,
    /// `CLIENT_NICKNAME` (string).
    Nickname,
    /// `CLIENT_FLAG_TALKING` (integer).
    FlagTalking,
    /// `CLIENT_INPUT_MUTED` (integer).
    InputMuted,
    /// `CLIENT_OUTPUT_MUTED` (integer).
    OutputMuted,
    /// `CLIENT_INPUT_DEACTIVATED` (integer): the local microphone switch.
    InputDeactivated,
    /// `CLIENT_META_DATA` (string, up to 4096 bytes).
    MetaData,
    /// `CLIENT_IS_MUTED` (integer): muted locally by us.
    IsMuted,
}

impl ClientProperty {
    /// The `size_t flag` TeamSpeak takes for this property.
    pub fn flag(self) -> usize {
        let property = match self {
            Self::UniqueIdentifier => sdk::ClientProperties::UniqueIdentifier,
            Self::Nickname => sdk::ClientProperties::Nickname,
            Self::FlagTalking => sdk::ClientProperties::FlagTalking,
            Self::InputMuted => sdk::ClientProperties::InputMuted,
            Self::OutputMuted => sdk::ClientProperties::OutputMuted,
            Self::InputDeactivated => sdk::ClientProperties::InputDeactivated,
            Self::MetaData => sdk::ClientProperties::MetaData,
            Self::IsMuted => sdk::ClientProperties::IsMuted,
        };
        property as usize
    }
}

/// The `enum ChannelProperties` flags TFAR reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelProperty {
    /// `CHANNEL_NAME` (string).
    Name,
}

impl ChannelProperty {
    /// The `size_t flag` TeamSpeak takes for this property.
    pub fn flag(self) -> usize {
        match self {
            Self::Name => sdk::ChannelProperties::Name as usize,
        }
    }
}

/// The `enum VirtualServerProperties` flags TFAR reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerProperty {
    /// `VIRTUALSERVER_UNIQUE_IDENTIFIER` (string).
    UniqueIdentifier,
    /// `VIRTUALSERVER_NAME` (string).
    Name,
}

impl ServerProperty {
    /// The `size_t flag` TeamSpeak takes for this property.
    pub fn flag(self) -> usize {
        let property = match self {
            Self::UniqueIdentifier => sdk::VirtualServerProperties::UniqueIdentifier,
            Self::Name => sdk::VirtualServerProperties::Name,
        };
        property as usize
    }
}

/// `enum PluginTargetMode`: who receives a `sendPluginCommand`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginCommandTarget<'clients> {
    /// `PluginCommandTarget_CURRENT_CHANNEL`: every client in our channel.
    CurrentChannel,
    /// `PluginCommandTarget_SERVER`: every client on the server.
    Server,
    /// `PluginCommandTarget_CLIENT`: the listed clients.
    Clients(&'clients [crate::ClientId]),
    /// `PluginCommandTarget_CURRENT_CHANNEL_SUBSCRIBED_CLIENTS`.
    CurrentChannelSubscribedClients,
}

impl PluginCommandTarget<'_> {
    /// The `int targetMode` TeamSpeak takes for this target.
    pub fn mode(self) -> i32 {
        let mode = match self {
            Self::CurrentChannel => sdk::PluginTargetMode::CurrentChannel,
            Self::Server => sdk::PluginTargetMode::Server,
            Self::Clients(_) => sdk::PluginTargetMode::Client,
            Self::CurrentChannelSubscribedClients => {
                sdk::PluginTargetMode::CurrentChannelSubscribedClients
            }
        };
        mode as i32
    }
}

/// `TS3_VECTOR`: a position or direction in TeamSpeak's 3D sound space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ts3Vector {
    /// X co-ordinate.
    pub x: f32,
    /// Y co-ordinate.
    pub y: f32,
    /// Z co-ordinate.
    pub z: f32,
}

impl Ts3Vector {
    /// The `ts3plugin-sys` value the 3D functions take.
    pub(crate) fn to_sdk(self) -> sdk::Ts3Vector {
        sdk::Ts3Vector {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
