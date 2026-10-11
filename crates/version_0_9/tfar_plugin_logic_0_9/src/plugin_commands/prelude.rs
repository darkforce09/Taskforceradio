//! The names a caller of `tfar_plugin_logic_0_9::plugin_commands` imports with
//! `use crate::plugin_commands::prelude::*;`, with the `RadioSubtype` a tangent command carries.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`;
//! the command names and metadata tags stay in [`crate::plugin_commands::plugin_command`] and [`crate::plugin_commands::metadata`]
//! so their short names keep their module.

pub use crate::plugin_commands::metadata::{
    DISABLED_METADATA, format_metadata, info_data_text, merge_metadata, metadata_block,
    metadata_says_playing,
};
pub use crate::plugin_commands::plugin_command::{PluginCommand, TangentKind};
pub use crate::plugin_commands::sender_filter::{PLUGIN_NAME_PREFIX, accepts_sender};
pub use tfar_foundation::arma_values::RadioSubtype;
