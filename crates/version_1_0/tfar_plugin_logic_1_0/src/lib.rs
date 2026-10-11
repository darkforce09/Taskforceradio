//! The logic of the TFAR 1.0 TeamSpeak plugin, one module per stage: [`plugin_settings`] holds what
//! `SETCFG` sets, [`antennas`] models the radio towers, [`game_commands`] parses what the game
//! sends, [`plugin_commands`] encodes what clients tell each other through TeamSpeak,
//! [`client_state`] holds every client's game state, [`radio_routing`] decides how the listener
//! hears a speaker, [`radio_effects`] shapes radio sound, and [`voice_pipeline`] runs it all on
//! each voice frame. `tfar_plugin_1_0` adds the TeamSpeak exports and threads.

pub mod antennas;
pub mod client_state;
pub mod game_commands;
pub mod plugin_commands;
pub mod plugin_settings;
pub mod radio_effects;
pub mod radio_routing;
pub mod voice_pipeline;
