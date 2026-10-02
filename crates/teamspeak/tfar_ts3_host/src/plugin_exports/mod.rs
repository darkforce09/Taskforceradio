//! The `ts3plugin_*` export layer: the [`Ts3Plugin`] trait, its argument and answer types, the
//! [`ts3_plugin!`](crate::ts3_plugin) macro and the glue the macro expands to calls of.

mod audio_exports_macro;
mod audio_glue;
mod event_arguments;
mod event_exports_macro;
mod event_glue;
pub mod export_glue;
mod export_macro;
mod metadata_glue;
mod plugin_answers;
mod plugin_memory;
mod plugin_trait;

pub use event_arguments::{
    CapturedVoiceData, ClientMove, Invoker, MixedPlaybackVoiceData, PluginCommand,
    PostProcessVoiceData, ServerError, ServerPermissionError,
};
pub use plugin_answers::{CommandHandling, ErrorDisplay, InfoItem, InitStatus};
pub use plugin_trait::Ts3Plugin;

#[cfg(test)]
#[path = "tests/export_macro.rs"]
mod export_macro_tests;

#[cfg(test)]
#[path = "tests/plugin_memory.rs"]
mod plugin_memory_tests;
