//! The error type of `tfar_game_link::client`.
//!
//! **Role:** names every way a connect, a handshake or a write of the game link client fails, with
//! the port, the frame kind or the mismatch a reader needs to act.
//!
//! **Position:** returned by [`GameLinkClient`](crate::client::GameLinkClient)'s `send_async` and
//! `send_focus`; `sync_request` folds these into a [`SyncOutcome`](crate::client::SyncOutcome).
//!
//! **Signals and state:** none; plain values.
//!
//! **Invariants:** an error never carries a game-facing string; the extension apps choose those.

use core::time::Duration;

use crate::protocol::{FrameKind, HelloMismatch};

/// Every failure `tfar_game_link::client` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A failed connect or a dropped session started the backoff, and it has not passed yet; no
    /// connect was attempted.
    #[error("the game link backoff is running for another {remaining:?}; no connect was attempted")]
    BackoffRunning {
        /// How long until the next connect attempt is allowed.
        remaining: Duration,
    },
    /// The TCP connect to the plugin's port failed or took longer than the connect timeout.
    #[error("connecting to the game link plugin on 127.0.0.1:{port} failed: {source}")]
    Connect {
        /// The port the connect went to.
        port: u16,
        /// The socket error.
        #[source]
        source: std::io::Error,
    },
    /// Setting up the connected socket (no-delay, timeouts, the reader's handle) failed.
    #[error("setting up the game link socket to port {port} failed at `{operation}`: {source}")]
    SocketSetup {
        /// The port of the connection.
        port: u16,
        /// The socket call that failed.
        operation: &'static str,
        /// The socket error.
        #[source]
        source: std::io::Error,
    },
    /// The plugin did not answer `HELLO` within the `HELLO` timeout.
    #[error("the game link plugin on port {port} sent no HELLO within {timeout:?}")]
    HelloTimeout {
        /// The port of the connection.
        port: u16,
        /// The `HELLO` timeout that passed.
        timeout: Duration,
    },
    /// The plugin closed the connection before its `HELLO` arrived (it refused ours).
    #[error("the game link plugin on port {port} closed the connection during the handshake")]
    ClosedDuringHandshake {
        /// The port of the connection.
        port: u16,
    },
    /// The plugin's `HELLO` does not match this extension's magic, protocol or flavour.
    #[error("the game link plugin on port {port} sent a HELLO that does not match: {mismatch}")]
    HelloRefused {
        /// The port of the connection.
        port: u16,
        /// What did not match.
        mismatch: HelloMismatch,
    },
    /// The plugin sent another frame before its `HELLO`.
    #[error("the game link plugin on port {port} sent {kind} before HELLO")]
    FrameBeforeHello {
        /// The port of the connection.
        port: u16,
        /// The kind of the frame that came first.
        kind: FrameKind,
    },
    /// A frame from the plugin did not decode, or one of ours did not encode.
    #[error("a game link frame failed to encode or decode: {source}")]
    Frame {
        /// The codec's error.
        #[from]
        source: crate::protocol::Error,
    },
    /// Writing a frame to the plugin failed or took longer than the write timeout; the session is
    /// dropped.
    #[error("writing a {kind} frame to the game link plugin failed: {source}")]
    Write {
        /// The kind of the frame being written.
        kind: FrameKind,
        /// The socket error.
        #[source]
        source: std::io::Error,
    },
    /// The session's reader thread could not be started.
    #[error("starting the game link reader thread failed: {source}")]
    ReaderThread {
        /// The error from the thread builder.
        #[source]
        source: std::io::Error,
    },
}

impl Error {
    /// Whether this is a write that ran out of the write timeout rather than a closed connection.
    pub fn is_write_timeout(&self) -> bool {
        matches!(
            self,
            Self::Write { source, .. }
                if matches!(source.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)
        )
    }
}

/// The result of a fallible `tfar_game_link::client` operation.
pub type Result<T> = core::result::Result<T, Error>;
