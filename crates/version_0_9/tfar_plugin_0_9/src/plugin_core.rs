//! `PluginCore`: the plugin's shared runtime, made in `ts3plugin_init` and shared by every
//! thread the plugin runs on.
//!
//! **Role:** hold TeamSpeak, the server connections, the game session, the dormant, in-game and
//! link flags, the radio sounds with the decoded WAV library, the HRIR sphere, the local
//! push-to-talk and the running web requests; and give the command paths their few shared helpers (lock a server's state,
//! refresh a channel, send mute requests, play a sound).
//!
//! **Position:** behind an `Arc` in `TfarPlugin`'s running state; the game link bridge, the
//! service thread, the push-to-talk release thread, the TeamSpeak event handlers and the audio
//! callbacks all reach the plugin through it.
//!
//! **Signals and state:** lock order, outermost first: the push-to-talk lock, then the server
//! map (`RwLock`), then one connection's state (`Mutex`); the game session mutex is taken alone
//! and released before any other. No TeamSpeak call is made while the server map or a state
//! lock is held. The audio callbacks `try_read` the map and wait for a state, as the original
//! waits on its critical section; the command threads hold a state only for short decisions, so
//! the wait is short, and a block never goes out unprocessed because a decision was running.
//!
//! **Invariants:** a dormant plugin (no game activity since the session began or since its last
//! game end) sends no mute, nickname, metadata, channel move or plugin command; the flags are
//! atomics, read without a lock; waking and returning to dormant are decided under the game
//! session lock, so a game message never races the return to dormant.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use parking_lot::{Mutex, RwLock};
use tfar_client_state_0_9::GameSession;
use tfar_hrtf::HrirLibrary;
use tfar_playback_mixer::WaveLibrary;
use tfar_radio_routing_0_9::{MuteDecision, is_serious_mode};
use tfar_ts3_host::{AudioDiagnostics, ClientId, ServerConnectionHandlerId};

use crate::background_web_requests::BackgroundWebRequests;
use crate::channel_snapshot::{ChannelSnapshot, host_client_id, read_channel_snapshot};
use crate::push_to_talk::PushToTalk;
use crate::radio_sounds::RadioSounds;
use crate::server_connection::{ServerConnection, ServerState};
use crate::team_speak::TeamSpeak;

#[cfg(test)]
#[path = "tests/plugin_core.rs"]
mod tests;

/// What `init` found on disk and in TeamSpeak's settings for the core.
#[derive(Clone)]
pub struct PluginResources {
    /// TeamSpeak's plugins folder, holding `radio-sounds/` and `tfar_hrir/`.
    pub plugin_folder: std::path::PathBuf,
    /// The HRIR sphere; `None` when it could not be loaded, which turns the HRTF off.
    pub hrir: Option<Arc<HrirLibrary>>,
    /// How long a push-to-talk release waits (TeamSpeak's push-to-talk delay).
    pub push_to_talk_delay: Duration,
}

impl core::fmt::Debug for PluginResources {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PluginResources")
            .field("plugin_folder", &self.plugin_folder)
            .field("has_hrir", &self.hrir.is_some())
            .field("push_to_talk_delay", &self.push_to_talk_delay)
            .finish()
    }
}

/// The plugin's shared runtime; see the module header.
pub struct PluginCore {
    team_speak: Arc<dyn TeamSpeak>,
    servers: RwLock<BTreeMap<ServerConnectionHandlerId, Arc<ServerConnection>>>,
    session: Mutex<GameSession>,
    is_awake: AtomicBool,
    is_in_game: AtomicBool,
    is_link_connected: AtomicBool,
    sounds: RadioSounds,
    waves: WaveLibrary,
    hrir: Option<Arc<HrirLibrary>>,
    push_to_talk: PushToTalk,
    web_requests: BackgroundWebRequests,
    audio_diagnostics: AudioDiagnostics,
}

impl core::fmt::Debug for PluginCore {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PluginCore")
            .field("is_dormant", &self.is_dormant())
            .field("is_in_game", &self.is_in_game())
            .field("is_link_connected", &self.is_link_connected())
            .finish_non_exhaustive()
    }
}

impl PluginCore {
    /// A dormant core over `team_speak` with `resources`, at `now`; the remote radio sounds are
    /// decoded here, off the audio thread, and a file that will not decode is logged.
    pub fn new(team_speak: Arc<dyn TeamSpeak>, resources: PluginResources, now: Instant) -> Self {
        let sounds = RadioSounds::new(&resources.plugin_folder);
        let waves = WaveLibrary::new();
        for error in waves.preload(sounds.remote_sounds()) {
            tracing::warn!(%error, "a remote radio sound could not be loaded");
        }
        Self {
            team_speak,
            servers: RwLock::new(BTreeMap::new()),
            session: Mutex::new(GameSession::new(now)),
            is_awake: AtomicBool::new(false),
            is_in_game: AtomicBool::new(false),
            is_link_connected: AtomicBool::new(false),
            sounds,
            waves,
            hrir: resources.hrir,
            push_to_talk: PushToTalk::new(resources.push_to_talk_delay),
            web_requests: BackgroundWebRequests::new(),
            audio_diagnostics: AudioDiagnostics::new(),
        }
    }

    /// The audio callbacks' lock-wait and skipped-block counters.
    pub fn audio_diagnostics(&self) -> &AudioDiagnostics {
        &self.audio_diagnostics
    }

    /// The update check and the tracking requests still to join at shutdown.
    pub fn web_requests(&self) -> &BackgroundWebRequests {
        &self.web_requests
    }

    /// TeamSpeak.
    pub fn team_speak(&self) -> &dyn TeamSpeak {
        &*self.team_speak
    }

    /// Whether the plugin is dormant: no game activity has reached it since the session began or
    /// since its last game end.
    pub fn is_dormant(&self) -> bool {
        !self.is_awake.load(Ordering::Acquire)
    }

    /// Marks the plugin awake: a message that counts as game activity arrived.
    pub fn wake_up(&self) {
        if !self.is_awake.swap(true, Ordering::AcqRel) {
            tracing::info!("the game is active; the plugin leaves its dormant state");
        }
    }

    /// Records game activity at `now` under the game session lock: wakes the plugin and stamps
    /// the session; true when this starts a game. A message that does not count as game activity
    /// changes nothing and returns false.
    pub fn record_game_activity(&self, counts_as_game_activity: bool, now: Instant) -> bool {
        let mut session = self.session.lock();
        if counts_as_game_activity {
            self.wake_up();
        }
        session.record_message(counts_as_game_activity, now)
    }

    /// Puts the plugin back to dormant after its game-end handling, unless game activity has
    /// started a new game meanwhile; decided under the game session lock, so a message that
    /// wakes the plugin is never lost.
    pub fn return_to_dormant(&self) {
        let session = self.session.lock();
        if !session.is_in_game() && self.is_awake.swap(false, Ordering::AcqRel) {
            tracing::info!("the game has ended; the plugin is dormant until the next game");
        }
    }

    /// Whether a game session runs.
    pub fn is_in_game(&self) -> bool {
        self.is_in_game.load(Ordering::Acquire)
    }

    /// Records whether a game session runs.
    pub fn set_in_game(&self, is_in_game: bool) {
        self.is_in_game.store(is_in_game, Ordering::Release);
    }

    /// Whether an Arma extension is connected over the game link.
    pub fn is_link_connected(&self) -> bool {
        self.is_link_connected.load(Ordering::Acquire)
    }

    /// Records whether an Arma extension is connected.
    pub fn set_link_connected(&self, is_connected: bool) {
        self.is_link_connected
            .store(is_connected, Ordering::Release);
    }

    /// The game session, for one decision; never held across another lock.
    pub fn session(&self) -> parking_lot::MutexGuard<'_, GameSession> {
        self.session.lock()
    }

    /// The radio sound files.
    pub fn sounds(&self) -> &RadioSounds {
        &self.sounds
    }

    /// The decoded remote radio sounds.
    pub fn waves(&self) -> &WaveLibrary {
        &self.waves
    }

    /// The HRIR sphere, when loaded.
    pub fn hrir(&self) -> Option<&HrirLibrary> {
        self.hrir.as_deref()
    }

    /// The local push-to-talk.
    pub fn push_to_talk(&self) -> &PushToTalk {
        &self.push_to_talk
    }

    /// The server map for the audio callbacks: `None` while a connection is being added or
    /// removed, so the callback passes its block through instead of waiting; such a block is
    /// counted in the audio diagnostics.
    pub fn try_servers(
        &self,
    ) -> Option<
        parking_lot::RwLockReadGuard<
            '_,
            BTreeMap<ServerConnectionHandlerId, Arc<ServerConnection>>,
        >,
    > {
        let servers = self.servers.try_read();
        if servers.is_none() {
            self.audio_diagnostics.record_skipped_block();
        }
        servers
    }

    /// `connection`'s state for an audio callback: taken at once when free, else waited for and
    /// counted in the audio diagnostics. Safe to wait: no holder calls TeamSpeak or waits on
    /// anything while it holds a state.
    pub fn lock_state_for_audio<'connection>(
        &self,
        connection: &'connection ServerConnection,
    ) -> parking_lot::MutexGuard<'connection, ServerState> {
        connection.state.try_lock().unwrap_or_else(|| {
            self.audio_diagnostics.record_contended_state_lock();
            connection.state.lock()
        })
    }

    /// The connection `server`, for a command thread.
    pub fn connection(&self, server: ServerConnectionHandlerId) -> Option<Arc<ServerConnection>> {
        self.servers.read().get(&server).cloned()
    }

    /// Every connection, for a command thread.
    pub fn connections(&self) -> Vec<Arc<ServerConnection>> {
        self.servers.read().values().cloned().collect()
    }

    /// Runs `decide` on the state of `server` under its lock; `None` when the plugin keeps no
    /// state for it. `decide` must not call TeamSpeak.
    pub fn with_state<Answer>(
        &self,
        server: ServerConnectionHandlerId,
        decide: impl FnOnce(&mut ServerState) -> Answer,
    ) -> Option<Answer> {
        let connection = self.connection(server)?;
        let mut state = connection.state.lock();
        Some(decide(&mut state))
    }

    /// Starts keeping state for `server`, replacing what was kept, and reads its channel.
    pub fn add_connection(&self, server: ServerConnectionHandlerId) {
        let snapshot = read_channel_snapshot(self.team_speak(), server);
        let connection = ServerConnection::new(server, snapshot.my_nickname.clone());
        connection.state.lock().apply_channel_snapshot(&snapshot);
        self.servers.write().insert(server, Arc::new(connection));
    }

    /// Stops keeping state for `server`.
    pub fn remove_connection(&self, server: ServerConnectionHandlerId) {
        let removed = self.servers.write().remove(&server);
        drop(removed);
    }

    /// Drops every connection's state.
    pub fn clear_connections(&self) {
        let removed = std::mem::take(&mut *self.servers.write());
        drop(removed);
    }

    /// Reads the local player's channel on `server` and takes it in (`updateNicknamesList`);
    /// returns the snapshot.
    pub fn refresh_channel(&self, server: ServerConnectionHandlerId) -> ChannelSnapshot {
        let snapshot = read_channel_snapshot(self.team_speak(), server);
        self.with_state(server, |state| state.apply_channel_snapshot(&snapshot));
        snapshot
    }

    /// Sends the requests of `decision`; a client TeamSpeak refuses to change is forgotten by the
    /// tracker, so a later decision asks again. Call it with no lock held.
    pub fn send_mute_decision(&self, server: ServerConnectionHandlerId, decision: &MuteDecision) {
        if decision.is_empty() {
            return;
        }
        let team_speak = self.team_speak();
        let mute: Vec<_> = decision.mute.iter().copied().map(host_client_id).collect();
        let unmute: Vec<_> = decision
            .unmute
            .iter()
            .copied()
            .map(host_client_id)
            .collect();
        let refused_mute = team_speak.mute_clients(server, &mute).err();
        let refused_unmute = team_speak.unmute_clients(server, &unmute).err();
        if let Some(error) = &refused_mute {
            tracing::warn!(%error, "TeamSpeak refused to mute clients");
        }
        if let Some(error) = &refused_unmute {
            tracing::warn!(%error, "TeamSpeak refused to unmute clients");
        }
        if refused_mute.is_some() || refused_unmute.is_some() {
            self.with_state(server, |state| {
                if refused_mute.is_some() {
                    decision
                        .mute
                        .iter()
                        .for_each(|client| state.mutes.forget(*client));
                }
                if refused_unmute.is_some() {
                    decision
                        .unmute
                        .iter()
                        .for_each(|client| state.mutes.forget(*client));
                }
            });
        }
    }

    /// Plays the WAV file at `path` through TeamSpeak, logging a failure.
    pub fn play_sound(&self, server: ServerConnectionHandlerId, path: &Path) {
        if let Err(error) = self.team_speak().play_wave_file(server, path) {
            tracing::warn!(%error, path = %path.display(), "a radio sound could not be played");
        }
    }
}

/// What a mute decision needs from TeamSpeak about one client, read before the server lock.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MuteFacts {
    /// TeamSpeak's talking flag of the client.
    pub is_talking_in_teamspeak: bool,
    /// The name of the channel the client sits in; empty when unknown.
    pub channel_name: String,
}

impl MuteFacts {
    /// Whether the client sits in the serious mode channel `configured_name`.
    pub fn is_in_serious_channel(&self, configured_name: &str) -> bool {
        is_serious_mode(&self.channel_name, configured_name)
    }
}

impl PluginCore {
    /// Reads the [`MuteFacts`] of `client` on `server`. Makes TeamSpeak calls: never under a
    /// lock.
    pub fn mute_facts_of(&self, server: ServerConnectionHandlerId, client: ClientId) -> MuteFacts {
        let team_speak = self.team_speak();
        MuteFacts {
            is_talking_in_teamspeak: team_speak.is_client_talking(server, client),
            channel_name: team_speak
                .channel_of_client(server, client)
                .and_then(|channel| team_speak.channel_name(server, channel))
                .unwrap_or_default(),
        }
    }
}
