//! The names a caller of `tfar_plugin_logic_1_0::plugin_commands` imports with `use crate::plugin_commands::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::plugin_commands::command_codec::{decode, encode};
pub use crate::plugin_commands::metadata::{
    DISABLED_METADATA, INFO_DATA_BUFFER_SIZE, METADATA_END_TAG, METADATA_START_TAG,
    PLAYING_METADATA_PREFIX, format_metadata, info_data_text, merge_metadata, metadata_block,
    metadata_says_playing,
};
pub use crate::plugin_commands::plugin_command::{
    PluginCommand, TangentBroadcast, TangentKind, TeamspeakClientId,
};
pub use crate::plugin_commands::sender_filter::{PLUGIN_NAME_PREFIX, accepts_sender};
