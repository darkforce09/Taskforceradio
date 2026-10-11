//! The 1.0 plugin-to-plugin commands sent through TeamSpeak, byte-compatible with the original.
//!
//! **Role:** the 1.0 plugin commands exchanged between TeamSpeak clients and the metadata block,
//! encoded byte for byte like the original so Windows players still interoperate.
//!
//! **Position:** `crates/version_1_0`, tier 2, depending on `tfar_arma_values`; used by
//! `tfar_plugin_1_0`.
//!
//! **Signals and state:** none; pure encoding, callable from any thread.
//!
//! **Invariants:** the encoded bytes match what the original 1.0 plugin sends; a tangent is read
//! by its own token layout, never by shifted indices.

mod command_codec;
mod error;
mod metadata;
mod plugin_command;
pub mod prelude;
mod sender_filter;

pub use command_codec::{decode, encode};
pub use error::{Error, Result};
pub use metadata::{
    DISABLED_METADATA, INFO_DATA_BUFFER_SIZE, METADATA_END_TAG, METADATA_START_TAG,
    PLAYING_METADATA_PREFIX, format_metadata, info_data_text, merge_metadata, metadata_block,
    metadata_says_playing,
};
pub use plugin_command::{PluginCommand, TangentBroadcast, TangentKind, TeamspeakClientId};
pub use sender_filter::{PLUGIN_NAME_PREFIX, accepts_sender};
