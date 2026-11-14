//! [`GameState`]: what the game reports, kept once for the whole plugin.
//!
//! **Role:** hold the local player's game data (radios, liveness, data frame, speakers, tangent)
//! and the radio towers, as the original keeps one `m_gameData` and one antenna manager per
//! plugin (`reference/tfar_1_0/ts/src/task_force_radio.hpp:143,178`), so a server connected
//! after the game started reads the running game.
//!
//! **Position:** owned by `runtime::PluginRuntime` behind one `parking_lot::Mutex`; written by
//! the game link's async worker and the game start and end, read by the TeamSpeak event
//! handlers, the service thread, the diagnostics and the audio callbacks.
//!
//! **Signals and state:** plain data behind the runtime's game lock. The lock order is: the
//! server registry (only to clone a state out), then the game lock, then one server lock; only
//! `PluginRuntime::lock_game_and_server` takes both, and nothing takes the game lock while a
//! server lock is held.
//!
//! **Invariants:** no TeamSpeak call happens, and no lock but one server lock is taken, while the
//! game lock is held.

use tfar_plugin_logic_1_0::antennas::AntennaManager;
use tfar_plugin_logic_1_0::client_state::prelude::GameData;

/// The game's state for the whole plugin; see the module header.
#[derive(Debug, Default)]
pub struct GameState {
    /// What the game reports about the local player.
    pub game: GameData,
    /// The radio towers the game reported.
    pub antennas: AntennaManager,
}

impl GameState {
    /// No game yet: the data frame invalid, no tower.
    pub fn new() -> Self {
        Self::default()
    }
}
