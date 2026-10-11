//! The extension side of the game link: a lazy, time-bounded localhost TCP client.
//!
//! **Role:** the extension side of the game link: [`GameLinkClient`] connects lazily to the plugin's
//! `127.0.0.1` port within the connect timeout, completes the `HELLO` handshake, sends `SYNC_REQ`,
//! `ASYNC` and `FOCUS` frames and waits a bounded time for the matching `SYNC_RESP`. It holds no
//! game-facing string: the extension apps map each [`SyncOutcome`] and [`Error`] to the reply their
//! original returned.
//!
//! **Position:** `crates/game_link`, tier 1, depending on `tfar_game_link_protocol`; used by
//! `tfar_extension_0_9`, `tfar_extension_1_0`, `tfar_ts3_mock`.
//!
//! **Signals and state:** one session at a time behind a mutex, used from Arma's calling thread; one
//! reader thread per session decodes the plugin's frames and publishes the latest `STATUS` needcfg
//! and the last-frame time in atomics and the `SYNC_RESP` frames through a channel.
//!
//! **Invariants:** a call blocks for at most the connect timeout, the `HELLO` timeout and the sync
//! timeout together (about 1.3 s with the defaults), and in steady state at most the sync timeout;
//! after a failed connect or a dropped session no connect is attempted until the backoff has passed;
//! a response whose id is not the awaited request's is discarded.

mod client_config;
mod error;
mod game_link_client;
pub mod prelude;
mod session;
mod session_reader;
mod sync_outcome;

pub use client_config::ClientConfig;
pub use error::{Error, Result};
pub use game_link_client::GameLinkClient;
pub use sync_outcome::SyncOutcome;
