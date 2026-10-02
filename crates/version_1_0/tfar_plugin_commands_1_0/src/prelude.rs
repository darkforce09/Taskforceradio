//! The names a caller of `tfar_plugin_commands_1_0` imports with `use tfar_plugin_commands_1_0::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::command_codec::{decode, encode};
pub use crate::crate_role::CRATE_ROLE;
pub use crate::metadata::{
    DISABLED_METADATA, INFO_DATA_BUFFER_SIZE, METADATA_END_TAG, METADATA_START_TAG,
    PLAYING_METADATA_PREFIX, format_metadata, info_data_text, merge_metadata, metadata_block,
    metadata_says_playing,
};
pub use crate::plugin_command::{PluginCommand, TangentBroadcast, TangentKind, TeamspeakClientId};
pub use crate::sender_filter::{PLUGIN_NAME_PREFIX, accepts_sender};
