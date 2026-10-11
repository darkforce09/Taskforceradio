//! The 0.9.12 plugin-to-plugin commands sent through TeamSpeak, byte-compatible with the original.
//!
//! **Role:** the 0.9.12 plugin commands exchanged between TeamSpeak clients, the sender filter,
//! and the metadata block, encoded byte for byte like the original so Windows players still
//! interoperate.
//!
//! **Position:** a module of `tfar_plugin_logic_0_9`, depending on `tfar_foundation::arma_values`; used by
//! `tfar_plugin_0_9`.
//!
//! **Signals and state:** none; pure encoding and decoding, callable from any thread.
//!
//! **Invariants:** the encoded bytes match what the original 0.9.12 plugin sends; decoding reads
//! every field by position with a count check, so a malformed command is an `Error`, never a
//! panic.

mod error;
pub mod metadata;
pub mod plugin_command;
pub mod prelude;
mod sender_filter;

pub use error::{Error, Result};
pub use metadata::{
    DISABLED_METADATA, format_metadata, info_data_text, merge_metadata, metadata_block,
    metadata_says_playing,
};
pub use plugin_command::{PluginCommand, TangentKind};
pub use sender_filter::{PLUGIN_NAME_PREFIX, accepts_sender};
