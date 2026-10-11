//! The names a caller of `tfar_ts3_host` imports with `use tfar_ts3_host::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::error::Ts3Error;
pub use crate::host_ids::{ChannelId, ClientId, ServerConnectionHandlerId};
pub use crate::host_values::{
    ChannelProperty, ClientProperty, ConnectStatus, LogLevel, PluginCommandTarget, ServerProperty,
    TalkStatus, Ts3Vector, Visibility,
};
pub use crate::logging::{LoggingGuard, install_logging, install_logging_at, print_to_current_tab};
pub use crate::plugin_exports::{
    CapturedVoiceData, ClientMove, CommandHandling, ErrorDisplay, InfoItem, InitStatus, Invoker,
    MixedPlaybackVoiceData, PluginCommand, PostProcessVoiceData, ServerError,
    ServerPermissionError, Ts3Plugin,
};
pub use crate::ts3_functions::Ts3;
pub use crate::ts3_plugin;
