//! The names a caller of `tfar_game_link::server` imports with `use crate::server::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.
//! [`Flavour`] is re-exported so a plugin builds a [`ServerConfig`] without naming the protocol
//! crate.

pub use crate::protocol::Flavour;
pub use crate::server::game_link_handler::GameLinkHandler;
pub use crate::server::game_link_server::GameLinkServer;
pub use crate::server::server_config::ServerConfig;
