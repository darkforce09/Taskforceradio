//! The plugin side of the game link: a localhost TCP listener that answers the extension.
//!
//! **Role:** listens on `127.0.0.1`, on the normal port and optionally the `-tfdebug` port, checks
//! each connection's `HELLO`, keeps one session at a time, answers every `SYNC_REQ` through the
//! plugin's handler, queues `ASYNC` messages for one worker in arrival order, lets the plugin queue
//! its own async messages behind them, forwards `FOCUS` and sends the `STATUS` heartbeat and
//! needcfg pushes.
//!
//! **Position:** `crates/game_link`, tier 1, depending on `tfar_game_link_protocol`; used by
//! `tfar_plugin_0_9`, `tfar_plugin_1_0`, which pass in a [`GameLinkHandler`] and derive the
//! game-connected state from its callbacks and [`GameLinkServer::last_game_message`].
//!
//! **Signals and state:** three kinds of named threads: one accept thread per listen port, one
//! session thread per connection and the async worker; the shared state of [`GameLinkServer`] is
//! atomics and short `parking_lot` locks. [`GameLinkServer::shutdown`] stops and joins them all.
//!
//! **Invariants:** binds loopback only; async messages are delivered first in, first out, and a
//! `MISSIONEND` drops every queued message before it; a refused `HELLO` gets no reply; a frame that
//! does not decode closes its session; `HELLO` and `FOCUS` never count as game messages.

mod accept_loop;
mod async_worker;
mod error;
mod game_link_handler;
mod game_link_server;
mod link_state;
pub mod prelude;
mod server_config;
mod session_link;
mod session_loop;

pub use error::{Error, Result};
pub use game_link_handler::GameLinkHandler;
pub use game_link_server::GameLinkServer;
pub use server_config::{
    DEFAULT_BIND_RETRY_INTERVAL, DEFAULT_HELLO_TIMEOUT, DEFAULT_STATUS_INTERVAL,
    DEFAULT_WRITE_TIMEOUT, ServerConfig,
};
