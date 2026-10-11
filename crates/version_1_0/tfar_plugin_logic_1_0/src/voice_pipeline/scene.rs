//! `Scene`: the read-only view of one sender and the local player that planning works from.
//!
//! **Role:** gather, once per frame, what every planning step reads: both clients, the game
//! data, the settings, both positions extrapolated to now, the listener's view and whether the
//! two can speak to each other (`plugin.cpp:528-545`).
//!
//! **Position:** a module of `tfar_plugin_logic_1_0::voice_pipeline`; built by the frame plan and the DSP
//! needs while the session is borrowed immutably.
//!
//! **Signals and state:** none; borrows the session for the planning phase only.
//!
//! **Invariants:** a scene exists only when both the sender and the local player are known.

use std::time::Instant;

use crate::client_state::prelude::{ClientData, GameData, SessionState, TeamspeakClientId};
use crate::plugin_settings::PluginSettings;
use tfar_foundation::geometry::{Direction, Position};

/// One sender and the local player, read for planning.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Scene<'a> {
    /// The local player.
    pub(crate) listener: &'a ClientData,
    /// The sender.
    pub(crate) sender: &'a ClientData,
    /// The local player's game data.
    pub(crate) game: &'a GameData,
    /// The plugin settings.
    pub(crate) settings: &'a PluginSettings,
    /// The local player's position now.
    pub(crate) listener_position: Position,
    /// The local player's view direction.
    pub(crate) listener_view: Direction,
    /// The sender's position now.
    pub(crate) sender_position: Position,
    /// Both can speak (`shouldPlayerHear = clientData->canSpeak && canSpeak`).
    pub(crate) should_hear: bool,
    /// The sender is the local player (the captured-voice path).
    pub(crate) is_own_voice: bool,
    /// The time the scene is taken at.
    pub(crate) now: Instant,
}

impl<'a> Scene<'a> {
    /// The scene of `sender` in `session` at `now`; `None` when either client is unknown.
    pub(crate) fn of(
        session: &'a SessionState,
        settings: &'a PluginSettings,
        sender: TeamspeakClientId,
        now: Instant,
    ) -> Option<Self> {
        let listener = session.clients.own()?;
        let sender_data = session.clients.by_id(sender)?;
        Some(Self {
            listener,
            sender: sender_data,
            game: &session.game,
            settings,
            listener_position: listener.client_position(now),
            listener_view: listener.view_direction,
            sender_position: sender_data.client_position(now),
            should_hear: sender_data.can_speak && listener.can_speak,
            is_own_voice: listener.client_id() == sender,
            now,
        })
    }

    /// The straight distance between the two, in metres.
    pub(crate) fn distance(&self) -> f64 {
        self.listener_position.distance_to(self.sender_position)
    }

    /// The sender's voice range in metres, as a float.
    pub(crate) fn voice_volume(&self) -> f64 {
        f64::from(self.sender.voice_volume)
    }
}
