//! `GameLinkState`: whether the game is connected and in a game, from its message times.
//!
//! **Role:** turn the time of the game's last message into the connect and disconnect
//! transitions of `SharedMemoryHandler::isConnected`
//! (`reference/tfar_1_0/ts/src/SharedMemoryHandler.cpp:227-247`) and the in-game flag of
//! `onGameConnected` and `onGameDisconnected` (`task_force_radio.cpp:17-37`).
//!
//! **Position:** a module of `tfar_plugin_logic_1_0::client_state`; one per plugin, owned by the plugin app;
//! fed by the game link handler ([`GameLinkState::record_game_message`]) and polled by the game
//! link or service loop ([`GameLinkState::poll`]).
//!
//! **Signals and state:** the last message time, the connected flag, the time of the last
//! transition and the in-game flag; written by whichever thread polls, under the caller's lock.
//!
//! **Invariants:** the game counts as connected while its last message is younger than the
//! `pluginTimeout` setting; a transition is reported at most once per 500 ms, and a change
//! inside that window is reported when the window ends; a game starts only on a connect while
//! not in game and ends only on a disconnect while in game.

use std::time::{Duration, Instant};

use crate::plugin_settings::PluginSettings;

#[cfg(test)]
#[path = "tests/game_link_state.rs"]
mod tests;

/// The shortest time between two reported transitions (`SharedMemoryHandler.cpp:236`).
pub const TRANSITION_DEBOUNCE: Duration = Duration::from_millis(500);

/// A change of the game link's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameLinkTransition {
    /// The game link came up (`onGameConnected`).
    Connected {
        /// The plugin was not in a game, so a game starts: play `radio-sounds/on`, run the game
        /// start, set the data frame to the first.
        is_game_start: bool,
    },
    /// The game link went down (`onGameDisconnected`): unmute everyone, mark the player dead.
    Disconnected {
        /// The plugin was in a game, so the game ends: play `radio-sounds/off`, run the game
        /// end, ask the game for its settings again.
        is_game_end: bool,
    },
}

/// What one poll of the game link found.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameLinkPoll {
    /// The game is connected now (what `isConnected` returns).
    pub is_connected: bool,
    /// The transition to act on, if one is due.
    pub transition: Option<GameLinkTransition>,
}

/// The game link's connection and game state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GameLinkState {
    last_game_message_at: Option<Instant>,
    was_connected: bool,
    last_transition_at: Option<Instant>,
    is_in_game: bool,
}

impl GameLinkState {
    /// Disconnected, not in a game, no message yet.
    pub const fn new() -> Self {
        Self {
            last_game_message_at: None,
            was_connected: false,
            last_transition_at: None,
            is_in_game: false,
        }
    }

    /// The game sent a message at `now` (the game tick).
    pub const fn record_game_message(&mut self, now: Instant) {
        self.last_game_message_at = Some(now);
    }

    /// The game said it is leaving (`MISSIONEND`, the extension's `setGameDisconnected`): its
    /// last message is forgotten, so the next poll sees it gone.
    pub const fn forget_game_messages(&mut self) {
        self.last_game_message_at = None;
    }

    /// Whether the plugin is in a game (`currentlyInGame`).
    pub const fn is_in_game(&self) -> bool {
        self.is_in_game
    }

    /// Whether the game is connected at `now` under `settings`' `pluginTimeout`.
    pub fn is_connected(&self, now: Instant, settings: &PluginSettings) -> bool {
        let timeout = settings.plugin_timeout();
        self.last_game_message_at
            .is_some_and(|last| now.saturating_duration_since(last) < timeout)
    }

    /// Checks the link at `now` and reports the transition that is due, updating the in-game
    /// flag with it.
    pub fn poll(&mut self, now: Instant, settings: &PluginSettings) -> GameLinkPoll {
        let is_connected = self.is_connected(now, settings);
        let is_debouncing = self
            .last_transition_at
            .is_some_and(|last| now.saturating_duration_since(last) < TRANSITION_DEBOUNCE);
        if is_debouncing || is_connected == self.was_connected {
            return GameLinkPoll {
                is_connected,
                transition: None,
            };
        }
        self.was_connected = is_connected;
        self.last_transition_at = Some(now);
        let transition = if is_connected {
            let is_game_start = !self.is_in_game;
            self.is_in_game = true;
            GameLinkTransition::Connected { is_game_start }
        } else {
            let is_game_end = self.is_in_game;
            self.is_in_game = false;
            GameLinkTransition::Disconnected { is_game_end }
        };
        GameLinkPoll {
            is_connected,
            transition: Some(transition),
        }
    }
}
