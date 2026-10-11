//! `GameSession`: whether the local player is in a mission, from the game's message traffic.
//!
//! **Role:** turn the game messages into the in-game flag (the pipe thread's game start,
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:2056-2071`, and the service thread's 4 s timeout,
//! plugin.cpp:1992-2009), and apply a game's start and end to the server state.
//!
//! **Position:** the plugin app holds one `GameSession` beside its server states (the original's
//! `inGame` is global); the game link thread calls `record_message` after each message, the
//! service tick calls `check_timeout`; `ServerRadioData::start_game` and `end_game` apply the
//! transitions to the current connection.
//!
//! **Signals and state:** the in-game flag and the time of the last game activity; time comes
//! in as `now`.
//!
//! **Invariants:** the first message that counts as game activity starts a game whatever the
//! serious channel (serious mode only adds the channel move); a game ends after more than
//! [`GAME_ACTIVITY_TIMEOUT`] without activity; the original nickname is handed back exactly
//! once per game.

use std::time::{Duration, Instant};

use crate::client_state::server_radio_data::{INVALID_DATA_FRAME, ServerRadioData};

#[cfg(test)]
#[path = "tests/game_session.rs"]
mod tests;

/// How long the game may stay silent before the plugin counts the game as over
/// (`MILLIS_TO_EXPIRE`, plugin.cpp:55).
pub const GAME_ACTIVITY_TIMEOUT: Duration = Duration::from_secs(4);

/// The in-game flag and the last game activity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameSession {
    in_game: bool,
    last_game_activity: Instant,
}

/// What a timeout ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameTimeout {
    /// Whether a game was running and has now ended: the plugin plays the off sound and calls
    /// [`ServerRadioData::end_game`]. Either way it unmutes what it muted.
    pub ended_a_game: bool,
}

/// What the start of a game asks the plugin to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameStart {
    /// The serious mode channel to move into, when the mission names one.
    pub serious_channel: Option<SeriousChannel>,
}

/// A serious mode channel and its password.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeriousChannel {
    /// The channel name.
    pub name: String,
    /// The channel password.
    pub password: String,
}

/// What the end of a game asks the plugin to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameEnd {
    /// The nickname to set back, when the game renamed the local player.
    pub restore_nickname: Option<String>,
}

impl GameSession {
    /// No game, with the last activity at `now` (the original stamps it at load).
    pub fn new(now: Instant) -> Self {
        Self {
            in_game: false,
            last_game_activity: now,
        }
    }

    /// Whether a game is running.
    pub fn is_in_game(&self) -> bool {
        self.in_game
    }

    /// When the game last showed activity, or the session was made or last timed out.
    pub fn last_game_activity(&self) -> Instant {
        self.last_game_activity
    }

    /// Records a game message at `now`. One that counts as game activity (see
    /// `crate::game_commands::message_counts_as_game_activity`) stamps the activity and, when
    /// no game runs, starts one: the result is `true` exactly then, and the plugin plays the on
    /// sound and calls [`ServerRadioData::start_game`].
    pub fn record_message(&mut self, counts_as_game_activity: bool, now: Instant) -> bool {
        if !counts_as_game_activity {
            return false;
        }
        self.last_game_activity = now;
        let starts = !self.in_game;
        self.in_game = true;
        starts
    }

    /// Whether more than [`GAME_ACTIVITY_TIMEOUT`] has passed since the last activity.
    pub fn game_timed_out(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.last_game_activity) > GAME_ACTIVITY_TIMEOUT
    }

    /// On a timeout at `now`, restamps the activity, ends the game and says whether one ran;
    /// `None` when the game has not timed out.
    pub fn check_timeout(&mut self, now: Instant) -> Option<GameTimeout> {
        if !self.game_timed_out(now) {
            return None;
        }
        self.last_game_activity = now;
        let ended_a_game = self.in_game;
        self.in_game = false;
        Some(GameTimeout { ended_a_game })
    }
}

impl ServerRadioData {
    /// Applies the start of a game: the data frame restarts at 0. Returns the serious channel
    /// to move into when the mission names one.
    pub fn start_game(&mut self) -> GameStart {
        self.current_data_frame = 0;
        let serious_channel = (!self.serious_channel_name.is_empty()).then(|| SeriousChannel {
            name: self.serious_channel_name.clone(),
            password: self.serious_channel_password.clone(),
        });
        GameStart { serious_channel }
    }

    /// Applies the end of a game: the local player is not alive, the data frame is invalid, the
    /// serious channel is forgotten, and the original nickname is handed back to restore.
    pub fn end_game(&mut self) -> GameEnd {
        self.is_alive = false;
        self.current_data_frame = INVALID_DATA_FRAME;
        self.serious_channel_name.clear();
        self.serious_channel_password.clear();
        GameEnd {
            restore_nickname: self.my_original_nickname.take(),
        }
    }
}
