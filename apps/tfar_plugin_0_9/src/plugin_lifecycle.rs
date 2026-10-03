//! The plugin TeamSpeak loads: its metadata, `init` and `shutdown`, and every callback handed to
//! the shared core.
//!
//! **Role:** [`TfarPlugin`] implements [`Ts3Plugin`] with the original 0.9.12 plugin's metadata;
//! `init` installs the TeamSpeak log forwarder and starts a [`RunningPlugin`], `shutdown` stops
//! it and drops the log forwarder last; each event and audio callback goes to [`PluginCore`].
//!
//! **Position:** the `static` the crate root hands to `ts3_plugin!`.
//!
//! **Signals and state:** the running plugin and the logging guard behind one mutex, taken only
//! on TeamSpeak's main thread in `init` and `shutdown`; the core behind a `parking_lot::RwLock`
//! that the main-thread callbacks `read` and the audio callbacks only `try_read`, passing the
//! block through while `init` or `shutdown` holds it.
//!
//! **Invariants:** after `shutdown` no thread the plugin started is running and the core is
//! gone; a second `init` stops what the first one started before starting again; no audio
//! callback ever waits for a lock.

use std::ffi::CStr;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use parking_lot::RwLock;
use tfar_game_link_server::prelude::Flavour;
use tfar_ts3_host::{
    CapturedVoiceData, ChannelId, ClientId, ClientMove, CommandHandling, ConnectStatus,
    ErrorDisplay, InfoItem, InitStatus, Invoker, LoggingGuard, MixedPlaybackVoiceData,
    PluginCommand, PostProcessVoiceData, ServerConnectionHandlerId, ServerError, TalkStatus, Ts3,
    Ts3Plugin, install_logging, print_to_current_tab,
};

use crate::plugin_core::PluginCore;
use crate::running_plugin::RunningPlugin;
use crate::team_speak_events::server_error_display;

#[cfg(test)]
#[path = "tests/plugin_lifecycle.rs"]
mod tests;

/// The mod version this plugin serves.
pub const FLAVOUR: Flavour = Flavour::V0_9;

/// The channel name the plugin's lines carry in TeamSpeak's log.
pub const LOG_CHANNEL: &str = "tfar_plugin_0_9";

/// The 0.9.12 TeamSpeak plugin; see the module header.
pub struct TfarPlugin {
    running: Mutex<Option<Started>>,
    core: RwLock<Option<Arc<PluginCore>>>,
}

/// What `init` started, with the log forwarder that outlives it.
struct Started {
    plugin: RunningPlugin,
    /// `None` when the log forwarder could not be installed; the plugin runs without it.
    logging: Option<LoggingGuard>,
}

impl TfarPlugin {
    /// A plugin that has started nothing.
    pub const fn new() -> Self {
        Self {
            running: Mutex::new(None),
            core: RwLock::new(None),
        }
    }

    /// Whether `init` started the plugin and `shutdown` has not stopped it.
    pub fn is_running(&self) -> bool {
        self.lock_running().is_some()
    }

    /// Stops the plugin, joining its threads, then the log forwarder.
    fn stop(&self) {
        let started = self.lock_running().take();
        *self.core.write() = None;
        if let Some(Started { plugin, logging }) = started {
            plugin.stop();
            tracing::info!("stopped");
            drop(logging);
        }
    }

    fn lock_running(&self) -> MutexGuard<'_, Option<Started>> {
        // The guarded value is replaced whole, so a poisoned lock is still consistent (and with
        // `panic = "abort"` it never poisons).
        self.running.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The core, for a callback on TeamSpeak's main thread.
    fn core(&self) -> Option<Arc<PluginCore>> {
        self.core.read().clone()
    }

    /// Runs `callback` on the core from an audio thread, unless `init` or `shutdown` holds it.
    fn with_core_on_audio_thread(&self, callback: impl FnOnce(&PluginCore)) {
        if let Some(core) = self.core.try_read()
            && let Some(core) = core.as_deref()
        {
            callback(core);
        }
    }
}

impl Default for TfarPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for TfarPlugin {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("TfarPlugin")
            .field("is_running", &self.is_running())
            .finish_non_exhaustive()
    }
}

impl Ts3Plugin for TfarPlugin {
    const NAME: &'static CStr = c"Task Force Arma 3 Radio";
    const VERSION: &'static CStr = c"0.9.12";
    const AUTHOR: &'static CStr = c"[TF]Nkey";
    const DESCRIPTION: &'static CStr = c"Radio Addon for Arma 3";
    const INFO_TITLE: &'static CStr = c"Task Force Radio Status (0.9.12)";
    const COMMAND_KEYWORD: &'static CStr = c"";

    fn init(&self, ts3: &'static Ts3) -> InitStatus {
        self.stop();
        let logging = match install_logging(LOG_CHANNEL) {
            Ok(guard) => Some(guard),
            Err(error) => {
                // Without the forwarder the chat tab is the only place left to say so.
                let _ = print_to_current_tab(&format!(
                    "{LOG_CHANNEL}: the TeamSpeak log forwarder did not start: {error}"
                ));
                None
            }
        };
        match RunningPlugin::start(ts3) {
            Ok(plugin) => {
                *self.core.write() = Some(Arc::clone(plugin.core()));
                *self.lock_running() = Some(Started { plugin, logging });
                InitStatus::Success
            }
            Err(error) => {
                tracing::error!(%error, "the plugin did not start");
                drop(logging);
                InitStatus::Failure
            }
        }
    }

    fn shutdown(&self) {
        self.stop();
    }

    fn process_command(
        &self,
        _server: ServerConnectionHandlerId,
        _command: &str,
    ) -> CommandHandling {
        // The original answers 0 to every command; with no keyword TeamSpeak routes it none.
        CommandHandling::Handled
    }

    fn info_data(&self, server: ServerConnectionHandlerId, item: InfoItem) -> Option<String> {
        self.core()?.info_text(server, item)
    }

    fn on_connect_status_change(
        &self,
        server: ServerConnectionHandlerId,
        status: ConnectStatus,
        _error_number: u32,
    ) {
        if let Some(core) = self.core() {
            core.handle_connect_status(server, status);
        }
    }

    fn on_new_channel(&self, server: ServerConnectionHandlerId, _: ChannelId, _: ChannelId) {
        self.refresh_channel(server);
    }

    fn on_channel_move(
        &self,
        server: ServerConnectionHandlerId,
        _: ChannelId,
        _: ChannelId,
        _: Invoker<'_>,
    ) {
        self.refresh_channel(server);
    }

    fn on_update_channel(&self, server: ServerConnectionHandlerId, _: ChannelId) {
        self.refresh_channel(server);
    }

    fn on_update_client(&self, server: ServerConnectionHandlerId, _: ClientId, _: Invoker<'_>) {
        self.refresh_channel(server);
    }

    fn on_client_move(&self, movement: ClientMove<'_>) {
        self.client_moved(&movement);
    }

    fn on_client_move_timeout(&self, movement: ClientMove<'_>) {
        self.client_moved(&movement);
    }

    fn on_client_move_moved(&self, movement: ClientMove<'_>, _mover: Invoker<'_>) {
        self.client_moved(&movement);
    }

    fn on_client_kick_from_channel(&self, movement: ClientMove<'_>, _kicker: Invoker<'_>) {
        self.client_moved(&movement);
    }

    fn on_client_kick_from_server(&self, movement: ClientMove<'_>, _kicker: Invoker<'_>) {
        self.client_moved(&movement);
    }

    fn on_client_ban_from_server(&self, movement: ClientMove<'_>, _kicker: Invoker<'_>, _: u64) {
        self.client_moved(&movement);
    }

    fn on_server_error(&self, error: ServerError<'_>) -> ErrorDisplay {
        server_error_display(&error)
    }

    fn on_client_self_variable_update(
        &self,
        server: ServerConnectionHandlerId,
        flag: i32,
        _old_value: &str,
        new_value: &str,
    ) {
        if let Some(core) = self.core() {
            core.handle_own_variable_update(server, flag, new_value);
        }
    }

    fn on_talk_status_change(
        &self,
        server: ServerConnectionHandlerId,
        status: TalkStatus,
        _is_received_whisper: bool,
        client: ClientId,
    ) {
        if let Some(core) = self.core() {
            core.handle_talk_status_change(server, status, client);
        }
    }

    fn on_plugin_command(&self, command: PluginCommand<'_>) {
        if let Some(core) = self.core() {
            core.handle_plugin_command(&command, Instant::now());
        }
    }

    fn on_edit_post_process_voice_data(&self, mut data: PostProcessVoiceData<'_>) {
        self.with_core_on_audio_thread(|core| core.process_post_process_voice(&mut data));
    }

    fn on_edit_mixed_playback_voice_data(&self, mut data: MixedPlaybackVoiceData<'_>) {
        self.with_core_on_audio_thread(|core| core.process_mixed_playback(&mut data));
    }

    fn on_edit_captured_voice_data(&self, mut data: CapturedVoiceData<'_>) {
        self.with_core_on_audio_thread(|core| core.process_captured_voice(&mut data));
    }

    fn on_custom_3d_rolloff_calculation_client(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
        _distance: f32,
        volume: &mut f32,
    ) {
        self.with_core_on_audio_thread(|core| core.neutral_rolloff(server, client, volume));
    }
}

impl TfarPlugin {
    /// A channel or client event on `server`.
    fn refresh_channel(&self, server: ServerConnectionHandlerId) {
        if let Some(core) = self.core() {
            core.handle_channel_change(server);
        }
    }

    /// A client move, timeout, kick or ban.
    fn client_moved(&self, movement: &ClientMove<'_>) {
        if let Some(core) = self.core() {
            core.handle_client_move(movement);
        }
    }
}
