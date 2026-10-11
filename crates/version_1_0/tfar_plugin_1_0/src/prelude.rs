//! The names a caller of `tfar_plugin_1_0` imports with `use tfar_plugin_1_0::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::listen_port::{
    PORT_OVERRIDE_VARIABLE, server_config, server_config_from_environment,
};
pub use crate::plugin_lifecycle::{
    FLAVOUR, LOG_CHANNEL, PLUGIN_VERSION, SKIP_UPDATE_CHECK_VARIABLE, TfarPlugin,
};
pub use crate::runtime::PluginRuntime;
pub use crate::teamspeak_actions::TeamSpeakActions;
