//! [`PluginRuntime`]: the running 1.0 plugin, created in `ts3plugin_init` and stopped in
//! `ts3plugin_shutdown`.
//!
//! **Role:** own every piece of plugin state (the server states, the settings snapshot, the game
//! link state, the radio sounds and the HRIR sphere, the push-to-talk state, the link server, the
//! service thread, the debug window, the web requests, the game message log) and the helpers the
//! handlers share.
//!
//! **Position:** one per load, behind an `Arc` the plugin static publishes through an `ArcSwap`;
//! the game link handler and the service thread keep it alive until shutdown joins them.
//!
//! **Signals and state:** see the fields. The lock order is: the server registry (only to clone
//! a state out), then one server lock; every other lock (game link state, link server, service,
//! debug window, web requests, message log, warnings) is a leaf that is never held while a server
//! lock is taken. The push-to-talk lock is held across TeamSpeak calls, which is allowed because
//! no audio callback and no TeamSpeak callback takes it.
//!
//! **Invariants:** no TeamSpeak call while a server lock is held; nothing but queries reaches
//! TeamSpeak while the plugin is dormant ([`PluginRuntime::act`] answers `None`): before its first
//! game message and again after each game end.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;
use parking_lot::Mutex;
use tfar_client_state_1_0::prelude::{DspContext, GameLinkState, TeamspeakClientId};
use tfar_debug_window::prelude::DebugWindowHandle;
use tfar_game_link_server::prelude::GameLinkServer;
use tfar_plugin_settings_1_0::PluginSettings;
use tfar_ts3_host::{AudioDiagnostics, ServerConnectionHandlerId};
use tfar_voice_pipeline_1_0::prelude::{ensure_every_sender_dsp, ensure_sender_dsp};

use crate::audio::radio_sounds::RadioSounds;
use crate::diagnostics::message_ring::MessageRing;
use crate::push_to_talk::PushToTalk;
use crate::runtime::background_requests::BackgroundRequests;
use crate::runtime::server_state::{
    MAX_BLOCK_FRAMES, ServerRegistry, ServerState, SharedServerState,
};
use crate::service::service_thread::ServiceThread;
use crate::teamspeak_actions::TeamSpeakActions;

/// How many game messages the diagnostics dump keeps.
pub const GAME_MESSAGE_LOG_CAPACITY: usize = 256;

/// How long shutdown waits for the web requests it cancelled, all together. With the joins of the
/// link server, the service thread and the release thread (each within about one 100 ms poll) it
/// keeps `ts3plugin_shutdown` under its 2 s budget.
pub const WEB_REQUEST_JOIN_BUDGET: Duration = Duration::from_secs(1);

/// The plugin's state from `init` to `shutdown`; see the module header.
pub struct PluginRuntime {
    /// TeamSpeak.
    ts: &'static dyn TeamSpeakActions,
    /// The state of every established connection.
    pub servers: ServerRegistry,
    /// The settings, replaced whole on every `SETCFG`, read without a lock.
    settings: ArcSwap<PluginSettings>,
    /// Serialises the read-modify-write of [`PluginRuntime::update_settings`].
    settings_writer: Mutex<()>,
    /// Whether the game is connected and in a game.
    pub game_link: Mutex<GameLinkState>,
    /// The in-game flag of [`PluginRuntime::game_link`], for the audio callbacks.
    is_in_game: AtomicBool,
    /// A game message arrived since the plugin loaded or since its last game end: the plugin is
    /// not dormant.
    has_game_activity: AtomicBool,
    /// Shutdown has begun: no thread writes the own metadata any more.
    is_shutting_down: AtomicBool,
    /// Arma's window is in front, from the latest `FOCUS` frame.
    is_arma_focused: AtomicBool,
    /// The server tab in front, for the mixed-playback callback.
    front_server: AtomicU64,
    /// The game link server, `None` before it starts and after shutdown takes it.
    link_server: Mutex<Option<GameLinkServer>>,
    /// The radio sounds, the playback mixer and the HRIR sphere.
    pub sounds: RadioSounds,
    /// The tangent state, the voice-activation memory and the release thread.
    pub push_to_talk: PushToTalk,
    /// TeamSpeak's push-to-talk release delay, read once at init.
    push_to_talk_delay: Duration,
    /// The service thread.
    service: Mutex<Option<ServiceThread>>,
    /// The debug window, once `/tfar debug` opened it.
    pub debug_window: Mutex<Option<DebugWindowHandle>>,
    /// The latest game messages, for the diagnostics dump.
    pub game_messages: Mutex<MessageRing>,
    /// The minimum-plugin-version warning was shown.
    pub has_warned_minimum_version: AtomicBool,
    /// The nickname-too-short warning was shown.
    pub has_warned_short_nickname: AtomicBool,
    /// The update check and the tracking requests, cancelled and joined at shutdown.
    pub web_requests: BackgroundRequests,
    /// The seed of the next underwater dropout generator.
    dropout_seed: AtomicU64,
    /// When the plugin last asked to move into the serious channel.
    pub last_serious_move_at: Mutex<Option<Instant>>,
    /// The audio callbacks' lock waits and skipped blocks, logged by the service thread.
    pub audio_diagnostics: AudioDiagnostics,
}

impl core::fmt::Debug for PluginRuntime {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PluginRuntime")
            .field("is_in_game", &self.is_in_game())
            .field("is_dormant", &self.is_dormant())
            .finish_non_exhaustive()
    }
}

impl PluginRuntime {
    /// A runtime over `ts` with the loaded `sounds` and TeamSpeak's `push_to_talk_delay`; nothing
    /// runs yet.
    pub fn new(
        ts: &'static dyn TeamSpeakActions,
        sounds: RadioSounds,
        push_to_talk_delay: Duration,
    ) -> Arc<Self> {
        Arc::new(Self {
            ts,
            servers: ServerRegistry::new(),
            settings: ArcSwap::from_pointee(PluginSettings::default()),
            settings_writer: Mutex::new(()),
            game_link: Mutex::new(GameLinkState::new()),
            is_in_game: AtomicBool::new(false),
            has_game_activity: AtomicBool::new(false),
            is_shutting_down: AtomicBool::new(false),
            is_arma_focused: AtomicBool::new(false),
            front_server: AtomicU64::new(0),
            link_server: Mutex::new(None),
            sounds,
            push_to_talk: PushToTalk::new(),
            push_to_talk_delay,
            service: Mutex::new(None),
            debug_window: Mutex::new(None),
            game_messages: Mutex::new(MessageRing::new(GAME_MESSAGE_LOG_CAPACITY)),
            has_warned_minimum_version: AtomicBool::new(false),
            has_warned_short_nickname: AtomicBool::new(false),
            web_requests: BackgroundRequests::new(),
            audio_diagnostics: AudioDiagnostics::new(),
            dropout_seed: AtomicU64::new(0x5446_4152),
            last_serious_move_at: Mutex::new(None),
        })
    }

    /// TeamSpeak, for queries.
    pub fn ts(&self) -> &'static dyn TeamSpeakActions {
        self.ts
    }

    /// TeamSpeak for side effects: `None` while the plugin is dormant.
    pub fn act(&self) -> Option<&'static dyn TeamSpeakActions> {
        (!self.is_dormant()).then_some(self.ts)
    }

    /// No game message arrived since the plugin loaded or since its last game end (decision D25).
    pub fn is_dormant(&self) -> bool {
        !self.has_game_activity.load(Ordering::Acquire)
    }

    /// A game message arrived: the plugin wakes until its next game end.
    pub fn mark_game_activity(&self) {
        self.has_game_activity.store(true, Ordering::Release);
    }

    /// The game-end handling has finished: the plugin is dormant again until the next game
    /// message (decision D25).
    pub fn return_to_dormant(&self) {
        self.has_game_activity.store(false, Ordering::Release);
    }

    /// Whether shutdown has begun.
    pub fn is_shutting_down(&self) -> bool {
        self.is_shutting_down.load(Ordering::Acquire)
    }

    /// Whether a game is running.
    pub fn is_in_game(&self) -> bool {
        self.is_in_game.load(Ordering::Acquire)
    }

    /// Mirrors the game link state's in-game flag for the audio callbacks.
    pub fn set_in_game(&self, is_in_game: bool) {
        self.is_in_game.store(is_in_game, Ordering::Release);
    }

    /// Whether Arma's window is in front.
    pub fn is_arma_focused(&self) -> bool {
        self.is_arma_focused.load(Ordering::Acquire)
    }

    /// Records a `FOCUS` frame (or `false` when the session closed).
    pub fn set_arma_focused(&self, is_focused: bool) {
        self.is_arma_focused.store(is_focused, Ordering::Release);
    }

    /// The server tab in front.
    pub fn front_server(&self) -> ServerConnectionHandlerId {
        ServerConnectionHandlerId(self.front_server.load(Ordering::Acquire))
    }

    /// Records the server tab in front.
    pub fn set_front_server(&self, server: ServerConnectionHandlerId) {
        self.front_server.store(server.0, Ordering::Release);
    }

    /// TeamSpeak's push-to-talk release delay.
    pub const fn push_to_talk_delay(&self) -> Duration {
        self.push_to_talk_delay
    }

    /// The current settings.
    pub fn settings(&self) -> Arc<PluginSettings> {
        self.settings.load_full()
    }

    /// The current settings without touching the reference count; for the audio callbacks.
    pub fn settings_snapshot(&self) -> arc_swap::Guard<Arc<PluginSettings>> {
        self.settings.load()
    }

    /// Changes the settings with `change` and publishes the result; returns what `change`
    /// returned.
    pub fn update_settings<T>(&self, change: impl FnOnce(&mut PluginSettings) -> T) -> T {
        let _writer = self.settings_writer.lock();
        let mut settings = PluginSettings::clone(&self.settings.load());
        let answer = change(&mut settings);
        self.settings.store(Arc::new(settings));
        answer
    }

    /// The current server and its state, when it is established.
    pub fn current_server(&self) -> Option<(ServerConnectionHandlerId, SharedServerState)> {
        let server = self.ts.current_server();
        self.servers.get(server).map(|state| (server, state))
    }

    /// The state of `server`, created when it is new.
    pub fn server_state(&self, server: ServerConnectionHandlerId) -> SharedServerState {
        self.servers
            .get_or_insert(server, || ServerState::new(self.push_to_talk_delay))
    }

    /// Publishes the needcfg flag to the extension.
    pub fn set_needcfg(&self, needcfg: bool) {
        if let Some(server) = self.link_server.lock().as_ref() {
            server.set_needcfg(needcfg);
        }
    }

    /// Whether the link has a live session.
    pub fn is_link_connected(&self) -> bool {
        self.link_server
            .lock()
            .as_ref()
            .is_some_and(GameLinkServer::is_session_up)
    }

    /// Keeps the started link server until shutdown.
    pub fn attach_link_server(&self, server: GameLinkServer) {
        *self.link_server.lock() = Some(server);
    }

    /// Keeps the started service thread until shutdown.
    pub fn attach_service(&self, service: ServiceThread) {
        *self.service.lock() = Some(service);
    }

    /// Queues `text` (an async call without its `~`) on the game link's async worker behind
    /// every message already queued, as if the extension had sent it as an `ASYNC` frame; the
    /// sync `POS` answer schedules its apply this way. False, dropping `text`, before the link
    /// server starts and after shutdown takes it.
    pub fn queue_async_message(&self, text: &str) -> bool {
        self.link_server
            .lock()
            .as_ref()
            .is_some_and(|server| server.queue_async(text.as_bytes()))
    }

    /// What the DSP objects of the voice chain are built with.
    pub fn dsp_context(&self, settings: &PluginSettings) -> DspContext<'_> {
        DspContext {
            hrir_library: self.sounds.hrir_library(),
            object_interception_strength: settings.object_interception_strength(),
            dropout_seed: self.dropout_seed.fetch_add(1, Ordering::Relaxed),
            frames_per_block: MAX_BLOCK_FRAMES,
        }
    }

    /// Creates the DSP objects `sender`'s next frames need (off the audio thread, lock held).
    pub fn ensure_sender_dsp(
        &self,
        state: &mut ServerState,
        settings: &PluginSettings,
        sender: TeamspeakClientId,
    ) {
        let dsp = self.dsp_context(settings);
        let ServerState {
            session, antennas, ..
        } = state;
        if let Err(error) =
            ensure_sender_dsp(session, settings, antennas, sender, &dsp, Instant::now())
        {
            tracing::warn!(%error, "a DSP object of a sender could not be built");
        }
    }

    /// Creates the DSP objects every client's next frames need (lock held).
    pub fn ensure_every_sender_dsp(&self, state: &mut ServerState, settings: &PluginSettings) {
        let dsp = self.dsp_context(settings);
        let ServerState {
            session, antennas, ..
        } = state;
        if let Err(error) =
            ensure_every_sender_dsp(session, settings, antennas, &dsp, Instant::now())
        {
            tracing::warn!(%error, "a DSP object could not be built");
        }
    }

    /// Stops everything `init` started, in the shutdown order: the web requests are cancelled,
    /// the game ends if it runs, the metadata says disabled and is cleared, every mute is
    /// lifted, then the link server, the service thread, the tangent release thread and the
    /// debug window stop and are joined, and the web requests are joined within
    /// [`WEB_REQUEST_JOIN_BUDGET`].
    pub fn shut_down(self: &Arc<Self>) {
        self.is_shutting_down.store(true, Ordering::Release);
        self.web_requests.cancel_all();
        if self.is_in_game() {
            self.end_game();
            self.set_in_game(false);
        }
        self.clear_own_metadata();
        for (server, _) in self.servers.all() {
            self.unmute_all(server);
        }
        let link_server = self.link_server.lock().take();
        if let Some(link_server) = link_server {
            link_server.shutdown();
        }
        let service = self.service.lock().take();
        if let Some(service) = service {
            service.stop();
        }
        self.push_to_talk.stop();
        let window = self.debug_window.lock().take();
        if let Some(window) = window
            && let Err(error) = window.shutdown()
        {
            tracing::warn!(%error, "the debug window did not stop cleanly");
        }
        let report = self.web_requests.cancel_and_join(WEB_REQUEST_JOIN_BUDGET);
        tracing::debug!(
            joined = report.joined,
            left_running = report.left_running,
            "web requests stopped"
        );
        self.sounds.release_finished();
        self.servers.clear();
    }

    /// A weak handle for the threads that must not keep the runtime alive on their own.
    pub fn downgrade(self: &Arc<Self>) -> Weak<Self> {
        Arc::downgrade(self)
    }
}
