//! [`Ts3Plugin`]: what a TFAR plugin app implements; [`ts3_plugin!`](crate::ts3_plugin) turns it
//! into the `ts3plugin_*` exports TeamSpeak looks up.
//!
//! **Role:** one trait method per export TFAR needs, each with a no-op default, so a plugin writes
//! only the callbacks it uses. Raw C arguments never reach this trait: IDs are newtypes, text is
//! lossy `&str`, sample buffers are bounded slices.
//!
//! **Position:** the plugin-facing contract of `plugin_exports`; the glue files call it, the plugin
//! apps implement it.
//!
//! **Signals and state:** the plugin value is a `static` the app owns, shared across threads, so
//! the trait needs `Sync` and every method takes `&self`. TeamSpeak calls the `on_edit_*` methods
//! and the rolloff method on its audio threads; everything else arrives on its main thread.
//!
//! **Invariants:** the audio methods must not block and must not call TeamSpeak; the string
//! constants are NUL-terminated `'static` data, so the pointers handed to TeamSpeak never dangle.

use std::ffi::CStr;

use super::event_arguments::{
    CapturedVoiceData, ClientMove, Invoker, MixedPlaybackVoiceData, PluginCommand,
    PostProcessVoiceData, ServerError, ServerPermissionError,
};
use super::plugin_answers::{CommandHandling, ErrorDisplay, InfoItem, InitStatus};
use crate::{ChannelId, ClientId, ConnectStatus, ServerConnectionHandlerId, Ts3};

/// A TeamSpeak plugin: metadata as constants and one method per callback TFAR handles.
pub trait Ts3Plugin: Sync + 'static {
    /// `ts3plugin_name`: the name in TeamSpeak's plugin list.
    const NAME: &'static CStr;
    /// `ts3plugin_version`.
    const VERSION: &'static CStr;
    /// `ts3plugin_author`.
    const AUTHOR: &'static CStr;
    /// `ts3plugin_description`.
    const DESCRIPTION: &'static CStr;
    /// `ts3plugin_infoTitle`: the heading of the plugin's block in the info panel.
    const INFO_TITLE: &'static CStr = Self::NAME;
    /// `ts3plugin_commandKeyword`: the word after `/` that routes chat commands to the plugin.
    const COMMAND_KEYWORD: &'static CStr = c"";

    /// `ts3plugin_init`: the plugin starts; `ts3` is the installed function table.
    fn init(&self, _ts3: &'static Ts3) -> InitStatus {
        InitStatus::Success
    }

    /// `ts3plugin_shutdown`: the plugin stops; every thread it started must be joined here.
    fn shutdown(&self) {}

    /// `ts3plugin_registerPluginID`: TeamSpeak's ID for the plugin, already stored in [`Ts3`].
    fn register_plugin_id(&self, _plugin_id: &str) {}

    /// `ts3plugin_processCommand`: a `/<keyword> …` chat command.
    fn process_command(
        &self,
        _server: ServerConnectionHandlerId,
        _command: &str,
    ) -> CommandHandling {
        CommandHandling::NotHandled
    }

    /// `ts3plugin_currentServerConnectionChanged`: another server tab came to the front.
    fn current_server_connection_changed(&self, _server: ServerConnectionHandlerId) {}

    /// `ts3plugin_infoData`: the text for the info panel of `item`; `None` shows nothing.
    fn info_data(&self, _server: ServerConnectionHandlerId, _item: InfoItem) -> Option<String> {
        None
    }

    /// `onConnectStatusChangeEvent`.
    fn on_connect_status_change(
        &self,
        _server: ServerConnectionHandlerId,
        _status: ConnectStatus,
        _error_number: u32,
    ) {
    }

    /// `onNewChannelEvent`: a channel became known while connecting.
    fn on_new_channel(
        &self,
        _server: ServerConnectionHandlerId,
        _channel: ChannelId,
        _parent: ChannelId,
    ) {
    }

    /// `onNewChannelCreatedEvent`: a client created a channel.
    fn on_new_channel_created(
        &self,
        _server: ServerConnectionHandlerId,
        _channel: ChannelId,
        _parent: ChannelId,
        _invoker: Invoker<'_>,
    ) {
    }

    /// `onDelChannelEvent`.
    fn on_delete_channel(
        &self,
        _server: ServerConnectionHandlerId,
        _channel: ChannelId,
        _invoker: Invoker<'_>,
    ) {
    }

    /// `onChannelMoveEvent`: a channel got a new parent.
    fn on_channel_move(
        &self,
        _server: ServerConnectionHandlerId,
        _channel: ChannelId,
        _new_parent: ChannelId,
        _invoker: Invoker<'_>,
    ) {
    }

    /// `onUpdateChannelEvent`.
    fn on_update_channel(&self, _server: ServerConnectionHandlerId, _channel: ChannelId) {}

    /// `onUpdateChannelEditedEvent`: a client edited a channel.
    fn on_update_channel_edited(
        &self,
        _server: ServerConnectionHandlerId,
        _channel: ChannelId,
        _invoker: Invoker<'_>,
    ) {
    }

    /// `onUpdateClientEvent`: a client's variables changed.
    fn on_update_client(
        &self,
        _server: ServerConnectionHandlerId,
        _client: ClientId,
        _invoker: Invoker<'_>,
    ) {
    }

    /// `onClientMoveEvent`: a client switched channel by itself, joined or left.
    fn on_client_move(&self, _movement: ClientMove<'_>) {}

    /// `onClientMoveTimeoutEvent`: a client timed out; `message` is the timeout message.
    fn on_client_move_timeout(&self, _movement: ClientMove<'_>) {}

    /// `onClientMoveMovedEvent`: `mover` moved a client.
    fn on_client_move_moved(&self, _movement: ClientMove<'_>, _mover: Invoker<'_>) {}

    /// `onClientKickFromChannelEvent`.
    fn on_client_kick_from_channel(&self, _movement: ClientMove<'_>, _kicker: Invoker<'_>) {}

    /// `onClientKickFromServerEvent`.
    fn on_client_kick_from_server(&self, _movement: ClientMove<'_>, _kicker: Invoker<'_>) {}

    /// `onClientBanFromServerEvent`; `ban_seconds` is the ban length, `0` for permanent.
    fn on_client_ban_from_server(
        &self,
        _movement: ClientMove<'_>,
        _kicker: Invoker<'_>,
        _ban_seconds: u64,
    ) {
    }

    /// `onServerErrorEvent`.
    fn on_server_error(&self, _error: ServerError<'_>) -> ErrorDisplay {
        ErrorDisplay::Show
    }

    /// `onServerPermissionErrorEvent`.
    fn on_server_permission_error(&self, _error: ServerPermissionError<'_>) -> ErrorDisplay {
        ErrorDisplay::Show
    }

    /// `onClientSelfVariableUpdateEvent`: one of our own variables changed; `flag` is the
    /// `enum ClientProperties` value.
    fn on_client_self_variable_update(
        &self,
        _server: ServerConnectionHandlerId,
        _flag: i32,
        _old_value: &str,
        _new_value: &str,
    ) {
    }

    /// `onPluginCommandEvent`: a plugin command from another client (API 26 form).
    fn on_plugin_command(&self, _command: PluginCommand<'_>) {}

    /// `onEditPostProcessVoiceDataEvent` (audio thread): a speaker's voice after TeamSpeak's
    /// processing.
    fn on_edit_post_process_voice_data(&self, _data: PostProcessVoiceData<'_>) {}

    /// `onEditMixedPlaybackVoiceDataEvent` (audio thread): the final playback mix.
    fn on_edit_mixed_playback_voice_data(&self, _data: MixedPlaybackVoiceData<'_>) {}

    /// `onEditCapturedVoiceDataEvent` (audio thread): our own microphone.
    fn on_edit_captured_voice_data(&self, _data: CapturedVoiceData<'_>) {}

    /// `onCustom3dRolloffCalculationClientEvent` (audio thread): the 3D attenuation of `client` at
    /// `distance`; write the gain into `volume`, which arrives holding TeamSpeak's own value.
    fn on_custom_3d_rolloff_calculation_client(
        &self,
        _server: ServerConnectionHandlerId,
        _client: ClientId,
        _distance: f32,
        _volume: &mut f32,
    ) {
    }
}
