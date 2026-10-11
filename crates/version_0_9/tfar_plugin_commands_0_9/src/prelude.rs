//! The names a caller of `tfar_plugin_commands_0_9` imports with
//! `use tfar_plugin_commands_0_9::prelude::*;`, with the `RadioSubtype` a tangent command carries.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`;
//! the command names and metadata tags stay in [`crate::plugin_command`] and [`crate::metadata`]
//! so their short names keep their module.

pub use crate::metadata::{
    DISABLED_METADATA, format_metadata, info_data_text, merge_metadata, metadata_block,
    metadata_says_playing,
};
pub use crate::plugin_command::{PluginCommand, TangentKind};
pub use crate::sender_filter::{PLUGIN_NAME_PREFIX, accepts_sender};
pub use tfar_arma_values::RadioSubtype;
