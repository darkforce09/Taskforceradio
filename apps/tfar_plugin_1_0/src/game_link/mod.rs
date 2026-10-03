//! The game link side of the plugin: the handler the link server calls, the sync answers, the
//! async messages, the local tangent and the settings.

pub mod async_commands;
pub mod link_handler;
pub mod local_tangent;
pub mod settings_apply;
pub mod sync_replies;

pub use link_handler::LinkHandler;
