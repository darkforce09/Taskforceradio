//! The settings of a [`GameLinkClient`](crate::GameLinkClient): the flavour, the port and every
//! timeout of the game link spec.

use core::time::Duration;

use tfar_game_link_protocol::Flavour;

#[cfg(test)]
#[path = "tests/client_config.rs"]
mod tests;

/// The flavour, the port and the timeouts of one [`GameLinkClient`](crate::GameLinkClient).
///
/// [`ClientConfig::new`] fills in the values of the game link spec's "Timeouts and backoff" table;
/// tests shorten them where a test would otherwise wait.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientConfig {
    /// The mod version this extension speaks, sent in `HELLO` and expected back.
    pub flavour: Flavour,
    /// The `127.0.0.1` port the plugin listens on (see [`tfar_game_link_protocol::port`]).
    pub port: u16,
    /// The longest a TCP connect to the plugin may take.
    pub connect_timeout: Duration,
    /// The longest the plugin's `HELLO` may take after the connect, else the connection is dropped.
    pub hello_timeout: Duration,
    /// After a failed connect or a dropped session, how long calls fail at once without a new
    /// connect attempt.
    pub backoff: Duration,
    /// The longest a sync request waits for its `SYNC_RESP`.
    pub sync_timeout: Duration,
    /// A session with no frame from the plugin for this long is dead (the heartbeat is 500 ms).
    pub session_dead_after: Duration,
    /// The longest one socket write may take, else the session is dropped.
    pub write_timeout: Duration,
}

impl ClientConfig {
    /// The spec's TCP connect timeout: 50 ms.
    pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_millis(50);
    /// The spec's `HELLO` timeout: 250 ms.
    pub const DEFAULT_HELLO_TIMEOUT: Duration = Duration::from_millis(250);
    /// The spec's backoff after a failure or a drop: 1 s.
    pub const DEFAULT_BACKOFF: Duration = Duration::from_secs(1);
    /// The spec's sync wait: 1000 ms.
    pub const DEFAULT_SYNC_TIMEOUT: Duration = Duration::from_millis(1000);
    /// The spec's dead-session time: 1000 ms without a plugin frame.
    pub const DEFAULT_SESSION_DEAD_AFTER: Duration = Duration::from_millis(1000);
    /// The spec's socket write timeout: 1000 ms.
    pub const DEFAULT_WRITE_TIMEOUT: Duration = Duration::from_millis(1000);

    /// The settings for `flavour` on `port`, with every timeout at the spec's value.
    pub const fn new(flavour: Flavour, port: u16) -> Self {
        Self {
            flavour,
            port,
            connect_timeout: Self::DEFAULT_CONNECT_TIMEOUT,
            hello_timeout: Self::DEFAULT_HELLO_TIMEOUT,
            backoff: Self::DEFAULT_BACKOFF,
            sync_timeout: Self::DEFAULT_SYNC_TIMEOUT,
            session_dead_after: Self::DEFAULT_SESSION_DEAD_AFTER,
            write_timeout: Self::DEFAULT_WRITE_TIMEOUT,
        }
    }
}
