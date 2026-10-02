//! The names a caller of `tfar_plugin_0_9` imports with `use tfar_plugin_0_9::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
pub use crate::game_link_stub::{
    PING_REPLY, PING_REQUEST, StubGameLinkHandler, UNKNOWN_COMMAND_REPLY,
};
pub use crate::listen_port::{
    DEBUG_PORT_VARIABLE, PORT_OVERRIDE_VARIABLE, server_config, server_config_from_environment,
};
pub use crate::plugin_lifecycle::{FLAVOUR, LOG_CHANNEL, TfarPlugin};
