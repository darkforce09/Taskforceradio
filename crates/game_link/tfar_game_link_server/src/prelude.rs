//! The names a caller of `tfar_game_link_server` imports with `use tfar_game_link_server::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.
//! [`Flavour`] is re-exported so a plugin builds a [`ServerConfig`] without naming the protocol
//! crate.

pub use crate::game_link_handler::GameLinkHandler;
pub use crate::game_link_server::GameLinkServer;
pub use crate::server_config::ServerConfig;
pub use tfar_game_link_protocol::Flavour;
