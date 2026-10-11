//! The logic of the TFAR 0.9.12 TeamSpeak plugin, one module per stage: [`game_commands`] parses
//! what the game sends, [`plugin_commands`] encodes what clients tell each other through
//! TeamSpeak, [`client_state`] holds every client's game state, [`radio_routing`] decides how the
//! listener hears a speaker, [`radio_effects`] shapes radio sound, and [`voice_pipeline`] runs it
//! all on each voice frame. `tfar_plugin_0_9` adds the TeamSpeak exports and threads.

pub mod client_state;
pub mod game_commands;
pub mod plugin_commands;
pub mod radio_effects;
pub mod radio_routing;
pub mod voice_pipeline;
