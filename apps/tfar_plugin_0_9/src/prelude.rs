//! The names a caller of `tfar_plugin_0_9` imports with `use tfar_plugin_0_9::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
pub use crate::listen_port::{
    PORT_OVERRIDE_VARIABLE, server_config, server_config_from_environment,
};
pub use crate::plugin_core::{PluginCore, PluginResources};
pub use crate::plugin_lifecycle::{FLAVOUR, LOG_CHANNEL, TfarPlugin};
pub use crate::running_plugin::SKIP_UPDATE_CHECK_VARIABLE;
pub use crate::team_speak::{TeamSpeak, TeamSpeakActions, TeamSpeakQueries, Ts3TeamSpeak};
