//! How a game link server listens and paces its session.

use std::time::Duration;

use tfar_game_link_protocol::{Flavour, port};

#[cfg(test)]
#[path = "tests/server_config.rs"]
mod tests;

/// The `STATUS` heartbeat period; the extension counts a session dead after 1000 ms of silence.
pub const DEFAULT_STATUS_INTERVAL: Duration = Duration::from_millis(500);
/// How long a busy port waits before the next bind attempt.
pub const DEFAULT_BIND_RETRY_INTERVAL: Duration = Duration::from_secs(5);
/// The socket write timeout; a write that takes longer drops the session.
pub const DEFAULT_WRITE_TIMEOUT: Duration = Duration::from_millis(1000);
/// How long a new connection may take to send its `HELLO` before it is dropped.
pub const DEFAULT_HELLO_TIMEOUT: Duration = Duration::from_millis(1000);

/// The settings a [`GameLinkServer`](crate::GameLinkServer) starts with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerConfig {
    /// The mod version served; a `HELLO` of another flavour is refused.
    pub flavour: Flavour,
    /// The `127.0.0.1` port to listen on; 0 lets the system pick one (tests read it back with
    /// [`GameLinkServer::local_addr`](crate::GameLinkServer::local_addr)).
    pub port: u16,
    /// The `STATUS` heartbeat period, [`DEFAULT_STATUS_INTERVAL`].
    pub status_interval: Duration,
    /// The wait between bind attempts on a busy port, [`DEFAULT_BIND_RETRY_INTERVAL`].
    pub bind_retry_interval: Duration,
    /// The socket write timeout, [`DEFAULT_WRITE_TIMEOUT`].
    pub write_timeout: Duration,
    /// The longest wait for a new connection's `HELLO`, [`DEFAULT_HELLO_TIMEOUT`].
    pub hello_timeout: Duration,
    /// The needcfg value before the first
    /// [`set_needcfg`](crate::GameLinkServer::set_needcfg): true for 1.0, which waits for SETCFG.
    /// The 0.9 server always sends false.
    pub initial_needcfg: bool,
}

impl ServerConfig {
    /// The settings of a server for `flavour` on `port`, with every period at its default.
    pub const fn new(flavour: Flavour, port: u16) -> Self {
        Self {
            flavour,
            port,
            status_interval: DEFAULT_STATUS_INTERVAL,
            bind_retry_interval: DEFAULT_BIND_RETRY_INTERVAL,
            write_timeout: DEFAULT_WRITE_TIMEOUT,
            hello_timeout: DEFAULT_HELLO_TIMEOUT,
            initial_needcfg: matches!(flavour, Flavour::V1_0),
        }
    }

    /// The settings of the plugin for `flavour` on its spec port; `debug` picks the port a
    /// `-tfdebug` Arma instance connects to.
    pub const fn for_plugin(flavour: Flavour, debug: bool) -> Self {
        Self::new(flavour, port(flavour, debug))
    }
}
