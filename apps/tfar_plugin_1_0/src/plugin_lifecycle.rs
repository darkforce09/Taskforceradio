//! The plugin TeamSpeak loads: its metadata, what `init` starts and `shutdown` stops, and the
//! routing of every TeamSpeak callback into the running [`PluginRuntime`].
//!
//! **Role:** [`TfarPlugin`] implements [`Ts3Plugin`] with the original 1.0 plugin's metadata.
//! `init` installs the log forwarder, loads the radio sounds and the HRIR sphere, reads the
//! push-to-talk delay, starts the game link server, the service thread and the update check;
//! `shutdown` ends the game, clears the metadata, lifts the mutes, stops every thread and drops
//! the log forwarder last.
//!
//! **Position:** the `static` the crate root hands to `ts3_plugin!`.
//!
//! **Signals and state:** the running runtime in an `ArcSwapOption`, which the audio callbacks
//! load without a lock; the logging guard behind a mutex taken only by `init` and `shutdown` on
//! TeamSpeak's main thread.
//!
//! **Invariants:** after `shutdown` no thread the plugin started runs, except an update check
//! still waiting for the network, which is logged; a second `init` stops the first one's work.

use std::ffi::CStr;
use std::sync::Arc;

use arc_swap::ArcSwapOption;
use parking_lot::Mutex;
use tfar_game_link_server::prelude::{Flavour, GameLinkServer};
use tfar_plugin_commands_1_0::prelude::info_data_text;
use tfar_ts3_host::{
    CapturedVoiceData, ChannelId, ClientId, ClientMove, CommandHandling, ConnectStatus,
    ErrorDisplay, InfoItem, InitStatus, Invoker, LoggingGuard, MixedPlaybackVoiceData,
    PluginCommand, PostProcessVoiceData, ServerConnectionHandlerId, ServerError, Ts3, Ts3Plugin,
    install_logging, print_to_current_tab,
};
use tfar_ts3_settings_db::prelude::{read_push_to_talk_delay, settings_database_path};
use tfar_web_requests::prelude::{UPDATE_AVAILABLE_MESSAGE, spawn_update_check};

use crate::audio::RadioSounds;
use crate::audio::voice_callbacks;
use crate::diagnostics::chat_commands::run_chat_command;
use crate::events::plugin_commands::{own_variable_changed, plugin_command_received};
use crate::game_link::LinkHandler;
use crate::listen_port::server_config_from_environment;
use crate::runtime::PluginRuntime;
use crate::service::ServiceThread;
use crate::teamspeak_actions::TeamSpeakActions;

#[cfg(test)]
#[path = "tests/plugin_lifecycle.rs"]
mod tests;

/// The mod version this plugin serves.
pub const FLAVOUR: Flavour = Flavour::V1_0;

/// The channel name the plugin's lines carry in TeamSpeak's log.
pub const LOG_CHANNEL: &str = "tfar_plugin_1_0";

/// The plugin version, as `TS_INFO VERSION` and the metadata report it.
pub const PLUGIN_VERSION: &str = "1.0.0.1";

/// The environment variable that, set to `1`, skips the update check; for test suites that load
/// the plugin without network.
pub const SKIP_UPDATE_CHECK_VARIABLE: &str = "TFAR_SKIP_UPDATE_CHECK";

/// The 1.0 TeamSpeak plugin; see the module header.
pub struct TfarPlugin {
    runtime: ArcSwapOption<PluginRuntime>,
    logging: Mutex<Option<LoggingGuard>>,
}

impl TfarPlugin {
    /// A plugin that has started nothing.
    pub const fn new() -> Self {
        Self {
            runtime: ArcSwapOption::const_empty(),
            logging: Mutex::new(None),
        }
    }

    /// Whether `init` started the plugin and `shutdown` has not stopped it.
    pub fn is_running(&self) -> bool {
        self.runtime.load().is_some()
    }

    /// Runs `action` with the running runtime, if there is one.
    fn with_runtime(&self, action: impl FnOnce(&Arc<PluginRuntime>)) {
        if let Some(runtime) = self.runtime.load().as_ref() {
            action(runtime);
        }
    }

    /// Stops the runtime, then the log forwarder.
    fn stop(&self) {
        let runtime = self.runtime.swap(None);
        if let Some(runtime) = runtime {
            runtime.shut_down();
            tracing::info!("stopped");
        }
        drop(self.logging.lock().take());
    }

    /// Starts everything `init` starts.
    fn start(&self, ts3: &'static Ts3) -> InitStatus {
        let ts: &'static dyn TeamSpeakActions = ts3;
        let (sounds, problems) = RadioSounds::load(ts.plugin_path());
        for problem in &problems {
            ts.print_to_current_tab(problem);
        }
        let push_to_talk_delay = ts
            .config_path()
            .map(|config| read_push_to_talk_delay(&settings_database_path(&config)))
            .map_or(std::time::Duration::ZERO, |read| match read {
                Ok(delay) => delay.effective(),
                Err(error) => {
                    tracing::info!(%error, "no push-to-talk delay from TeamSpeak's settings");
                    std::time::Duration::ZERO
                }
            });
        let runtime = PluginRuntime::new(ts, sounds, push_to_talk_delay);
        runtime.set_front_server(ts.current_server());
        let config = server_config_from_environment(FLAVOUR);
        let handler = Arc::new(LinkHandler::new(runtime.downgrade()));
        match GameLinkServer::start(config, handler) {
            Ok(server) => {
                tracing::info!(port = config.port, "started; the game link listens");
                runtime.attach_link_server(server);
            }
            Err(error) => {
                tracing::error!(%error, "the game link did not start");
                return InitStatus::Failure;
            }
        }
        match ServiceThread::start(runtime.downgrade()) {
            Ok(service) => runtime.attach_service(service),
            Err(error) => tracing::error!(%error, "the service thread did not start"),
        }
        if std::env::var(SKIP_UPDATE_CHECK_VARIABLE).as_deref() != Ok("1") {
            match spawn_update_check(PLUGIN_VERSION, || {
                let _ = print_to_current_tab(UPDATE_AVAILABLE_MESSAGE);
            }) {
                Ok(handle) => runtime.attach_update_check(handle),
                Err(error) => tracing::warn!(%error, "the update check did not start"),
            }
        }
        if let Ok(servers) = ts3.server_connection_handler_list() {
            for server in servers
                .into_iter()
                .filter(|server| ts.is_connected(*server))
            {
                runtime.server_established(server);
            }
        }
        self.runtime.store(Some(runtime));
        InitStatus::Success
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
    const NAME: &'static CStr = c"Task Force Arrowhead Radio";
    const VERSION: &'static CStr = c"1.0.0.1";
    const AUTHOR: &'static CStr = c"[TF]Nkey and Dedmen";
    const DESCRIPTION: &'static CStr = c"Radio Addon for Arma 3";
    const INFO_TITLE: &'static CStr = c"Task Force Radio Status";
    const COMMAND_KEYWORD: &'static CStr = c"tfar";

    fn init(&self, ts3: &'static Ts3) -> InitStatus {
        self.stop();
        match install_logging(LOG_CHANNEL) {
            Ok(guard) => *self.logging.lock() = Some(guard),
            Err(error) => {
                // Without the forwarder the chat tab is the only place left to say so.
                let _ = print_to_current_tab(&format!(
                    "{LOG_CHANNEL}: the TeamSpeak log forwarder did not start: {error}"
                ));
            }
        }
        #[cfg(feature = "tracy")]
        let _ = tracy_client::Client::start();
        let status = self.start(ts3);
        if status != InitStatus::Success {
            self.stop();
        }
        status
    }

    fn shutdown(&self) {
        self.stop();
    }

    fn process_command(
        &self,
        _server: ServerConnectionHandlerId,
        command: &str,
    ) -> CommandHandling {
        self.runtime
            .load()
            .as_ref()
            .map_or(CommandHandling::NotHandled, |runtime| {
                run_chat_command(runtime, command)
            })
    }

    fn current_server_connection_changed(&self, server: ServerConnectionHandlerId) {
        self.with_runtime(|runtime| runtime.set_front_server(server));
    }

    fn info_data(&self, server: ServerConnectionHandlerId, item: InfoItem) -> Option<String> {
        let InfoItem::Client(client) = item else {
            return None;
        };
        let guard = self.runtime.load();
        let runtime = guard.as_ref()?;
        let metadata = runtime.ts().client_metadata(server, client)?;
        let text = info_data_text(&metadata);
        (!text.is_empty()).then(|| text.to_owned())
    }

    fn on_connect_status_change(
        &self,
        server: ServerConnectionHandlerId,
        status: ConnectStatus,
        _error_number: u32,
    ) {
        self.with_runtime(|runtime| match status {
            ConnectStatus::ConnectionEstablished => {
                runtime.set_front_server(runtime.ts().current_server());
                runtime.server_established(server);
            }
            ConnectStatus::Disconnected => runtime.server_disconnected(server),
            _ => {}
        });
    }

    fn on_new_channel(&self, server: ServerConnectionHandlerId, _: ChannelId, _: ChannelId) {
        self.with_runtime(|runtime| runtime.refresh_channels(server));
    }

    fn on_new_channel_created(
        &self,
        server: ServerConnectionHandlerId,
        _: ChannelId,
        _: ChannelId,
        _: Invoker<'_>,
    ) {
        self.with_runtime(|runtime| runtime.refresh_channels(server));
    }

    fn on_delete_channel(&self, server: ServerConnectionHandlerId, _: ChannelId, _: Invoker<'_>) {
        self.with_runtime(|runtime| runtime.refresh_channels(server));
    }

    fn on_channel_move(
        &self,
        server: ServerConnectionHandlerId,
        _: ChannelId,
        _: ChannelId,
        _: Invoker<'_>,
    ) {
        self.with_runtime(|runtime| runtime.refresh_channels(server));
    }

    fn on_update_channel(&self, server: ServerConnectionHandlerId, _: ChannelId) {
        self.with_runtime(|runtime| runtime.refresh_channels(server));
    }

    fn on_update_channel_edited(
        &self,
        server: ServerConnectionHandlerId,
        _: ChannelId,
        _: Invoker<'_>,
    ) {
        self.with_runtime(|runtime| runtime.refresh_channels(server));
    }

    fn on_update_client(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
        _: Invoker<'_>,
    ) {
        self.with_runtime(|runtime| runtime.client_updated(server, client));
    }

    fn on_client_move(&self, movement: ClientMove<'_>) {
        self.with_runtime(|runtime| runtime.client_moved(&movement));
    }

    fn on_client_move_timeout(&self, movement: ClientMove<'_>) {
        self.with_runtime(|runtime| runtime.client_moved(&movement));
    }

    fn on_client_move_moved(&self, movement: ClientMove<'_>, _: Invoker<'_>) {
        self.with_runtime(|runtime| runtime.client_moved(&movement));
    }

    fn on_client_kick_from_channel(&self, movement: ClientMove<'_>, _: Invoker<'_>) {
        self.with_runtime(|runtime| runtime.client_moved(&movement));
    }

    fn on_client_kick_from_server(&self, movement: ClientMove<'_>, _: Invoker<'_>) {
        self.with_runtime(|runtime| runtime.client_moved(&movement));
    }

    fn on_client_ban_from_server(&self, movement: ClientMove<'_>, _: Invoker<'_>, _: u64) {
        self.with_runtime(|runtime| runtime.client_moved(&movement));
    }

    fn on_server_error(&self, error: ServerError<'_>) -> ErrorDisplay {
        if error.return_code.is_empty() {
            ErrorDisplay::Show
        } else {
            ErrorDisplay::Suppress
        }
    }

    fn on_client_self_variable_update(
        &self,
        server: ServerConnectionHandlerId,
        flag: i32,
        _old_value: &str,
        new_value: &str,
    ) {
        self.with_runtime(|runtime| own_variable_changed(runtime, server, flag, new_value));
    }

    fn on_plugin_command(&self, command: PluginCommand<'_>) {
        self.with_runtime(|runtime| {
            plugin_command_received(
                runtime,
                command.server,
                command.plugin_name,
                command.command,
            );
        });
    }

    fn on_edit_post_process_voice_data(&self, mut data: PostProcessVoiceData<'_>) {
        if let Some(runtime) = self.runtime.load().as_ref() {
            voice_callbacks::post_process_voice(runtime, &mut data);
        }
    }

    fn on_edit_mixed_playback_voice_data(&self, mut data: MixedPlaybackVoiceData<'_>) {
        if let Some(runtime) = self.runtime.load().as_ref() {
            voice_callbacks::mixed_playback(runtime, &mut data);
        }
    }

    fn on_edit_captured_voice_data(&self, mut data: CapturedVoiceData<'_>) {
        if let Some(runtime) = self.runtime.load().as_ref() {
            voice_callbacks::captured_voice(runtime, &mut data);
        }
    }

    fn on_custom_3d_rolloff_calculation_client(
        &self,
        _server: ServerConnectionHandlerId,
        _client: ClientId,
        _distance: f32,
        volume: &mut f32,
    ) {
        if let Some(runtime) = self.runtime.load().as_ref() {
            voice_callbacks::custom_3d_rolloff(runtime, volume);
        }
    }
}
