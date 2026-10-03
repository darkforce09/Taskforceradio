//! The running plugin: its state per server connection, the game lifecycle, serious mode, the
//! auto-mute and the client roster.

pub mod client_roster;
pub mod game_lifecycle;
pub mod mute_updates;
pub mod plugin_runtime;
pub mod serious_channel;
pub mod server_state;

pub use plugin_runtime::PluginRuntime;
pub use server_state::{ServerRegistry, ServerState, SharedServerState};
