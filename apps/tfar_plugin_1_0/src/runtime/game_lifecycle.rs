//! The game link's connect and disconnect transitions and what they start and end: the `on` and
//! `off` sounds, the game start (serious channel, first data frame) and the game end (channel and
//! nickname back, mutes lifted, push-to-talk and voice activation restored, needcfg).
//!
//! **Role:** `onGameConnected`, `onGameDisconnected`, `onGameStart` and `onGameEnd` of
//! `reference/tfar_1_0/ts/src/task_force_radio.cpp:17-53`, driven by
//! `tfar_client_state_1_0::GameLinkState`, plus the own metadata of `updateUserStatusInfo`.
//!
//! **Position:** methods of `PluginRuntime`; called by the game link handler on every game
//! message and by the service thread every 100 ms.
//!
//! **Signals and state:** the game link state lock is held only for the poll; the transition is
//! acted on after it is released.
//!
//! **Invariants:** a game starts only on a connect while not in game and ends only on a
//! disconnect while in game; the metadata is written only while the plugin is not dormant, and
//! once shutdown begins no transition runs and only the shutdown clears the metadata.

use std::path::PathBuf;
use std::time::Instant;

use tfar_client_state_1_0::prelude::GameLinkTransition;
use tfar_plugin_commands_1_0::prelude::{DISABLED_METADATA, format_metadata, merge_metadata};
use tfar_ts3_host::ServerConnectionHandlerId;

use crate::plugin_lifecycle::PLUGIN_VERSION;
use crate::runtime::plugin_runtime::PluginRuntime;
use crate::runtime::server_state::host_client_id;

/// The sound of a game connecting, relative to the plugins folder, without `.wav`.
pub const GAME_ON_SOUND: &str = "radio-sounds/on";

/// The sound of a game disconnecting.
pub const GAME_OFF_SOUND: &str = "radio-sounds/off";

impl PluginRuntime {
    /// A game message arrived at `now`: the plugin wakes, the message counts for the game
    /// timeout, and a due transition runs before the message is applied, as the original's link
    /// thread checks `isConnected` before it reads.
    pub fn record_game_message(&self, now: Instant) {
        self.mark_game_activity();
        self.game_link.lock().record_game_message(now);
        self.poll_game_link(now);
    }

    /// `MISSIONEND`: the game's messages are forgotten, so the game disconnects (debounced).
    pub fn end_mission(&self, now: Instant) {
        self.game_link.lock().forget_game_messages();
        self.poll_game_link(now);
    }

    /// Checks the game link at `now` and runs the transition that is due.
    pub fn poll_game_link(&self, now: Instant) {
        if self.is_shutting_down() {
            return;
        }
        let settings = self.settings();
        let poll = self.game_link.lock().poll(now, &settings);
        match poll.transition {
            Some(GameLinkTransition::Connected { is_game_start }) => {
                tracing::info!(is_game_start, "the game connected");
                if is_game_start {
                    self.set_in_game(true);
                    self.play_lifecycle_sound(GAME_ON_SOUND);
                    if let Some((server, state)) = self.current_server() {
                        state.lock().session.start_game();
                        self.move_to_serious_channel(server, true);
                    }
                    self.update_own_metadata();
                }
            }
            Some(GameLinkTransition::Disconnected { is_game_end }) => {
                tracing::info!(is_game_end, "the game disconnected");
                let server = self.ts().current_server();
                self.unmute_all(server);
                if is_game_end {
                    self.play_lifecycle_sound(GAME_OFF_SOUND);
                    self.end_game();
                    self.update_settings(
                        tfar_plugin_settings_1_0::PluginSettings::mark_needs_refresh,
                    );
                    self.set_needcfg(true);
                }
                self.set_in_game(false);
                if let Some((_, state)) = self.current_server() {
                    state.lock().session.end_game();
                }
                self.update_own_metadata();
            }
            None => {}
        }
    }

    /// `onGameEnd`: back to the original channel and nickname, every mute lifted, and the
    /// microphone and voice activation as they were before the last transmission.
    pub fn end_game(&self) {
        let server = self.ts().current_server();
        self.move_from_serious_channel(server);
        self.reset_own_nickname(server);
        self.unmute_all(server);
        if let Some(ts) = self.act() {
            self.push_to_talk.restore_on_game_end(ts, server);
        }
    }

    /// Restores the nickname saved at the first rename of the game.
    pub fn reset_own_nickname(&self, server: ServerConnectionHandlerId) {
        let Some(state) = self.servers.get(server) else {
            return;
        };
        let original = state.lock().original_nickname.clone();
        let (Some(original), Some(ts)) = (original, self.act()) else {
            return;
        };
        if ts.set_own_nickname(server, &original) {
            let mut state = state.lock();
            state.original_nickname = None;
            if let Some(own) = state.session.clients.own_client_id() {
                state.session.clients.client_renamed(own, &original);
            }
        }
    }

    /// Unmutes every client this plugin muted on `server`.
    pub fn unmute_all(&self, server: ServerConnectionHandlerId) {
        let Some(state) = self.servers.get(server) else {
            return;
        };
        let muted = state.lock().mutes.unmute_all();
        if muted.is_empty() {
            return;
        }
        let clients: Vec<_> = muted.into_iter().map(host_client_id).collect();
        if let Some(ts) = self.act() {
            ts.unmute_clients(server, &clients);
        }
    }

    /// Writes the own TFAR metadata block on the current server: connected, playing, versions.
    pub fn update_own_metadata(&self) {
        if self.is_shutting_down() {
            return;
        }
        let block = format_metadata(
            self.is_link_connected(),
            self.is_in_game(),
            PLUGIN_VERSION,
            self.settings().addon_version(),
        );
        self.write_own_metadata(&block);
    }

    /// The shutdown metadata: the disabled notice, then no TFAR block at all.
    pub fn clear_own_metadata(&self) {
        self.write_own_metadata(DISABLED_METADATA);
        self.write_own_metadata("");
    }

    /// Merges `block` into the own metadata of the current server, keeping text outside the tags.
    fn write_own_metadata(&self, block: &str) {
        let Some(ts) = self.act() else {
            return;
        };
        let server = ts.current_server();
        if !ts.is_connected(server) {
            return;
        }
        let Some(own) = ts.own_client_id(server) else {
            return;
        };
        let existing = ts.client_metadata(server, own).unwrap_or_default();
        let merged = merge_metadata(&existing, block);
        if merged != existing {
            ts.set_own_metadata(server, &merged);
        }
    }

    /// Plays `radio-sounds/on` or `off` through TeamSpeak.
    fn play_lifecycle_sound(&self, sound: &str) {
        let Some(ts) = self.act() else {
            return;
        };
        let Some(path) = self.sounds.sound_path(sound) else {
            return;
        };
        ts.play_wave_file(ts.current_server(), &path);
    }
}

/// `sound` (relative, without `.wav`) under `plugin_path`.
pub fn wave_path(plugin_path: &std::path::Path, sound: &str) -> PathBuf {
    plugin_path.join(format!("{sound}.wav"))
}
