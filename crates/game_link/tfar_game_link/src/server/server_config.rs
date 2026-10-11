//! How a game link server listens and paces its session.
//!
//! **Role:** [`ServerConfig`]: the flavour served, the one or two loopback ports listened on, the
//! heartbeat, retry and timeout periods, and the needcfg value before the first push.
//!
//! **Position:** built by the plugin app (or a test) and handed to
//! [`GameLinkServer::start`](crate::server::GameLinkServer::start); read by every thread of the server.
//!
//! **Signals and state:** a plain `Copy` value; no state.
//!
//! **Invariants:** [`ServerConfig::listen_ports`] yields the normal port first, then the debug
//! port when one is set; both ports feed the one session slot of the server.

use std::time::Duration;

use crate::protocol::{Flavour, port};

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

/// The settings a [`GameLinkServer`](crate::server::GameLinkServer) starts with.
///
/// The server listens on [`port`](Self::port) and, when set, on
/// [`debug_port`](Self::debug_port) too: each port has its own accept thread and its own bind
/// retry, and both feed one session slot, so the newest valid `HELLO` on either port wins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerConfig {
    /// The mod version served; a `HELLO` of another flavour is refused.
    pub flavour: Flavour,
    /// The normal `127.0.0.1` port to listen on; 0 lets the system pick one (tests read it back
    /// with [`GameLinkServer::local_addr`](crate::server::GameLinkServer::local_addr)).
    pub port: u16,
    /// The second `127.0.0.1` port, the one a `-tfdebug` Arma instance connects to; `None`
    /// listens on [`port`](Self::port) alone. 0 lets the system pick one (tests read it back with
    /// [`GameLinkServer::local_addrs`](crate::server::GameLinkServer::local_addrs)).
    pub debug_port: Option<u16>,
    /// The `STATUS` heartbeat period, [`DEFAULT_STATUS_INTERVAL`].
    pub status_interval: Duration,
    /// The wait between bind attempts on a busy port, [`DEFAULT_BIND_RETRY_INTERVAL`].
    pub bind_retry_interval: Duration,
    /// The socket write timeout, [`DEFAULT_WRITE_TIMEOUT`].
    pub write_timeout: Duration,
    /// The longest wait for a new connection's `HELLO`, [`DEFAULT_HELLO_TIMEOUT`].
    pub hello_timeout: Duration,
    /// The needcfg value before the first
    /// [`set_needcfg`](crate::server::GameLinkServer::set_needcfg): true for 1.0, which waits for SETCFG.
    /// The 0.9 server always sends false.
    pub initial_needcfg: bool,
}

impl ServerConfig {
    /// The settings of a server for `flavour` on the one port `port`, with every period at its
    /// default.
    pub const fn new(flavour: Flavour, port: u16) -> Self {
        Self {
            flavour,
            port,
            debug_port: None,
            status_interval: DEFAULT_STATUS_INTERVAL,
            bind_retry_interval: DEFAULT_BIND_RETRY_INTERVAL,
            write_timeout: DEFAULT_WRITE_TIMEOUT,
            hello_timeout: DEFAULT_HELLO_TIMEOUT,
            initial_needcfg: matches!(flavour, Flavour::V1_0),
        }
    }

    /// The settings of the plugin for `flavour` on one spec port; `debug` picks the port a
    /// `-tfdebug` Arma instance connects to.
    pub const fn for_plugin(flavour: Flavour, debug: bool) -> Self {
        Self::new(flavour, port(flavour, debug))
    }

    /// The settings of the plugin for `flavour` on both spec ports: the normal one in
    /// [`port`](Self::port) and the `-tfdebug` one in [`debug_port`](Self::debug_port).
    pub const fn for_plugin_on_both_ports(flavour: Flavour) -> Self {
        Self::new(flavour, port(flavour, false)).with_debug_port(port(flavour, true))
    }

    /// These settings with [`debug_port`](Self::debug_port) set to `debug_port`.
    #[must_use]
    pub const fn with_debug_port(self, debug_port: u16) -> Self {
        Self {
            debug_port: Some(debug_port),
            ..self
        }
    }

    /// Every port the server listens on: [`port`](Self::port) first, then
    /// [`debug_port`](Self::debug_port) when it is set.
    pub fn listen_ports(&self) -> impl Iterator<Item = u16> + use<> {
        [Some(self.port), self.debug_port].into_iter().flatten()
    }
}
