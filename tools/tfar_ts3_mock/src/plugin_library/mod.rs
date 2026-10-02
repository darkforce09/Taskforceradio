//! The loaded plugin: its library, its exports and its metadata.

pub(crate) mod loaded_plugin;
pub(crate) mod plugin_exports;

pub use loaded_plugin::{PluginLibrary, PluginMetadata};
pub use plugin_exports::{
    ClientMoveExport, InvokedChannelExport, InvokedChannelUpdateExport, InvokedClientMoveExport,
    NumberExport, PluginExports, TextExport,
};
