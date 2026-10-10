//! Serious mode in TeamSpeak: the channel cache, whether the local client is in the serious
//! channel, and the moves into it and back out.
//!
//! **Role:** `moveToSeriousChannel`, `moveFromSeriousChannel`, the channel name cache and
//! `checkIfSeriousModeEnabled` (`reference/tfar_1_0/ts/src/Teamspeak.cpp:77-87`, `:207-243`,
//! `:518-524`); the channel is chosen by `tfar_radio_routing_1_0::serious_mode_channel`.
//!
//! **Position:** methods of `PluginRuntime`; called on game start, every 4 s in game by the
//! service thread, on reconnect, on a `serious_channelName` setting, and on channel events.
//!
//! **Signals and state:** the server state's channel cache, own channel, original channel and
//! serious-mode flag, written under its lock; TeamSpeak is called after the lock is released.
//!
//! **Invariants:** a move happens only while Arma is in front, unless `moveWhileTabbedOut`; a
//! serious-mode change recomputes every client's mute.

use std::time::{Duration, Instant};

use tfar_radio_routing_1_0::{ChannelId as RoutingChannelId, serious_mode_channel};
use tfar_ts3_host::{ChannelId, ServerConnectionHandlerId};

use crate::runtime::plugin_runtime::PluginRuntime;
use crate::runtime::server_state::ServerState;

/// The shortest time between two moves the `serious_channelName` setting asks for, so TeamSpeak
/// has updated the own channel before the next look (`task_force_radio.cpp:79`).
pub const SERIOUS_MOVE_DEBOUNCE: Duration = Duration::from_millis(10);

/// The channel `state`'s serious channel setting names, from its channel cache.
pub fn serious_channel(state: &ServerState, configured_name: &str) -> Option<ChannelId> {
    serious_mode_channel(
        configured_name,
        state
            .channels
            .iter()
            .map(|(id, name)| (RoutingChannelId(id.0), name.as_str())),
    )
    .map(|channel| ChannelId(channel.0))
}

impl PluginRuntime {
    /// Reads `server`'s channels and the own channel into the cache, then re-checks serious mode.
    pub fn refresh_channels(&self, server: ServerConnectionHandlerId) {
        let ts = self.ts();
        let channels = ts.channels(server);
        let own_channel = ts
            .own_client_id(server)
            .and_then(|own| ts.channel_of_client(server, own));
        if let Some(state) = self.servers.get(server) {
            let mut state = state.lock();
            state.channels = channels;
            state.own_channel = own_channel;
        }
        self.check_serious_mode(server);
    }

    /// Recomputes whether the local client is in the serious channel; on a change, every mute.
    pub fn check_serious_mode(&self, server: ServerConnectionHandlerId) {
        let Some(state) = self.servers.get(server) else {
            return;
        };
        let settings = self.settings();
        let has_changed = {
            let mut state = state.lock();
            let channel = serious_channel(&state, settings.serious_channel_name());
            let is_serious = channel.is_some() && channel == state.own_channel;
            let has_changed = state.is_serious_mode != is_serious;
            state.is_serious_mode = is_serious;
            has_changed
        };
        if has_changed {
            tracing::info!(server = server.0, "serious mode changed");
            self.refresh_every_mute(server);
        }
    }

    /// Moves the local client into the serious channel when Arma is in front (or
    /// `moveWhileTabbedOut`), the channel exists and the client is elsewhere; saves the channel
    /// it leaves. `is_game_start` is false for the setting-driven move, which is debounced.
    pub fn move_to_serious_channel(&self, server: ServerConnectionHandlerId, is_game_start: bool) {
        let settings = self.settings();
        if !settings.move_while_tabbed_out() && !self.is_arma_focused() {
            return;
        }
        if !is_game_start {
            let mut last = self.last_serious_move_at.lock();
            let now = Instant::now();
            if last.is_some_and(|at| now.saturating_duration_since(at) <= SERIOUS_MOVE_DEBOUNCE) {
                return;
            }
            *last = Some(now);
        }
        let Some(ts) = self.act() else {
            return;
        };
        let Some(state) = self.servers.get(server) else {
            return;
        };
        let own_channel = ts
            .own_client_id(server)
            .and_then(|own| ts.channel_of_client(server, own));
        let target = {
            let mut state = state.lock();
            let Some(target) = serious_channel(&state, settings.serious_channel_name()) else {
                return;
            };
            if own_channel == Some(target) {
                return;
            }
            state.original_channel = own_channel;
            target
        };
        ts.move_own_client(server, target, settings.serious_channel_password());
    }

    /// Moves the local client back into the channel saved by the last move into the serious
    /// channel, with no password, and forgets it.
    pub fn move_from_serious_channel(&self, server: ServerConnectionHandlerId) {
        let Some(state) = self.servers.get(server) else {
            return;
        };
        let Some(original) = state.lock().original_channel.take() else {
            return;
        };
        let Some(ts) = self.act() else {
            return;
        };
        let own_channel = ts
            .own_client_id(server)
            .and_then(|own| ts.channel_of_client(server, own));
        if own_channel != Some(original) {
            ts.move_own_client(server, original, "");
        }
    }
}
