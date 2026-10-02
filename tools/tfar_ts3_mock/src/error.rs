//! The error type of `tfar_ts3_mock`.
//!
//! **Role:** [`Error`]: every failure of loading, starting and driving a plugin, each carrying what
//! a test needs to act on (the path, the export, the init answer, the ID, the block shape).
//!
//! **Position:** returned by every fallible function of the crate; the binary prints it.
//!
//! **Signals and state:** plain values; no state.
//!
//! **Invariants:** a failure is a value, never a panic; the loader and file system errors keep
//! their source.

use std::path::PathBuf;

use tfar_ts3_host::{ChannelId, ClientId, ServerConnectionHandlerId};

/// Every failure `tfar_ts3_mock` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The plugin file could not be loaded.
    #[error("cannot load the plugin library {}: {source}", .path.display())]
    LoadLibrary {
        /// The file asked for.
        path: PathBuf,
        /// The loader's reason.
        #[source]
        source: libloading::Error,
    },
    /// The plugin lacks an export TeamSpeak requires.
    #[error("the plugin {} does not export the required `{symbol}`: {source}", .path.display())]
    MissingRequiredSymbol {
        /// The plugin file.
        path: PathBuf,
        /// The missing export.
        symbol: &'static str,
        /// The loader's reason.
        #[source]
        source: libloading::Error,
    },
    /// The plugin was built for another plugin API, so TeamSpeak would refuse it.
    #[error("the plugin reports plugin API {found}; the mock hosts API {expected}")]
    ApiVersionMismatch {
        /// The API the mock hosts.
        expected: i32,
        /// The API `ts3plugin_apiVersion` answered.
        found: i32,
    },
    /// `ts3plugin_init` failed, so TeamSpeak would unload the plugin.
    #[error("ts3plugin_init answered {code}, so TeamSpeak would unload the plugin")]
    InitFailed {
        /// The answer: `1` for failure, `-2` for failure without a message.
        code: i32,
    },
    /// A driver needs an export the plugin does not have.
    #[error("the plugin does not export `{symbol}`, which this driver calls")]
    MissingExport {
        /// The missing export.
        symbol: &'static str,
    },
    /// A driver needs a current server and no server is connected.
    #[error("no server is connected, so there is no current server")]
    NoCurrentServer,
    /// The server is not one of the fake client's tabs.
    #[error("the fake client has no server {}", .server.0)]
    UnknownServer {
        /// The server asked for.
        server: ServerConnectionHandlerId,
    },
    /// The client is not on the current server.
    #[error("the current server has no client {}", .client.0)]
    UnknownClient {
        /// The client asked for.
        client: ClientId,
    },
    /// The channel is not on the current server.
    #[error("the current server has no channel {}", .channel.0)]
    UnknownChannel {
        /// The channel asked for.
        channel: ChannelId,
    },
    /// Every client ID of the server is taken.
    #[error("the server {} has no free client ID left", .server.0)]
    ClientIdsExhausted {
        /// The full server.
        server: ServerConnectionHandlerId,
    },
    /// A voice block whose length is not a whole number of frames of `channels` samples.
    #[error("{sample_count} samples are not whole frames of {channels} channels")]
    InvalidVoiceBlock {
        /// The samples given.
        sample_count: usize,
        /// The channels given.
        channels: usize,
    },
    /// A text argument holds a NUL byte, which a C string cannot carry.
    #[error("the {argument} holds a NUL byte")]
    InteriorNul {
        /// Which argument.
        argument: &'static str,
    },
    /// A temporary folder for the plugin or config path could not be made.
    #[error("cannot create the temporary folder {}: {source}", .path.display())]
    TemporaryFolder {
        /// The folder.
        path: PathBuf,
        /// The file system's reason.
        #[source]
        source: std::io::Error,
    },
}

/// The result of a fallible `tfar_ts3_mock` operation.
pub type Result<T> = core::result::Result<T, Error>;
