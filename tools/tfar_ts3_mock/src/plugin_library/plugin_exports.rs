//! [`PluginExports`]: every `ts3plugin_*` symbol a TFAR plugin exports, resolved from the loaded
//! library as typed function pointers.
//!
//! **Role:** the plugin side of the boundary: one field per export of the SDK's `plugin.h` at
//! API 26 that `tfar_ts3_host::ts3_plugin!` generates, required ones as plain pointers and
//! optional ones as `Option`.
//!
//! **Position:** filled by [`PluginLibrary::open`](crate::PluginLibrary::open); the drivers of
//! [`MockHost`](crate::MockHost) call through it, and tests may call any export directly.
//!
//! **Signals and state:** plain function pointers, valid while the [`PluginLibrary`] that
//! resolved them stays loaded.
//!
//! **Invariants:** each field's type is the export's C signature argument for argument; the
//! pointers are `unsafe` to call, since the plugin is foreign code reading raw pointers.
//!
//! [`PluginLibrary`]: crate::PluginLibrary

use std::ffi::{c_char, c_int, c_uint, c_void};

use libloading::Library;
use ts3plugin_sys::ts3functions::Ts3Functions;

/// An export that returns a static C string (`ts3plugin_name` and the like).
pub type TextExport = unsafe extern "C" fn() -> *const c_char;
/// An export that returns an `int` (`ts3plugin_apiVersion` and the like).
pub type NumberExport = unsafe extern "C" fn() -> c_int;
/// `ts3plugin_onClientMoveEvent` and `ts3plugin_onClientMoveTimeoutEvent`.
pub type ClientMoveExport = unsafe extern "C" fn(u64, u16, u64, u64, c_int, *const c_char);
/// The moved and kicked exports: a move plus the invoker's ID, name, unique ID and message.
pub type InvokedClientMoveExport = unsafe extern "C" fn(
    u64,
    u16,
    u64,
    u64,
    c_int,
    u16,
    *const c_char,
    *const c_char,
    *const c_char,
);
/// A channel event with its invoker (`ts3plugin_onNewChannelCreatedEvent`,
/// `ts3plugin_onChannelMoveEvent`): server, channel, parent, invoker ID, name, unique ID.
pub type InvokedChannelExport =
    unsafe extern "C" fn(u64, u64, u64, u16, *const c_char, *const c_char);
/// A channel event without a parent: server, channel, invoker ID, name, unique ID.
pub type InvokedChannelUpdateExport =
    unsafe extern "C" fn(u64, u64, u16, *const c_char, *const c_char);

/// Every `ts3plugin_*` export the mock knows, resolved from one plugin library.
#[derive(Debug, Clone, Copy)]
#[allow(
    clippy::struct_field_names,
    reason = "each field is named after the SDK export it holds"
)]
pub struct PluginExports {
    /// `ts3plugin_name` (required).
    pub name: TextExport,
    /// `ts3plugin_version` (required).
    pub version: TextExport,
    /// `ts3plugin_apiVersion` (required).
    pub api_version: NumberExport,
    /// `ts3plugin_author` (required).
    pub author: TextExport,
    /// `ts3plugin_description` (required).
    pub description: TextExport,
    /// `ts3plugin_setFunctionPointers` (required).
    pub set_function_pointers: unsafe extern "C" fn(Ts3Functions),
    /// `ts3plugin_init` (required).
    pub init: NumberExport,
    /// `ts3plugin_shutdown` (required).
    pub shutdown: unsafe extern "C" fn(),
    /// `ts3plugin_offersConfigure`.
    pub offers_configure: Option<NumberExport>,
    /// `ts3plugin_registerPluginID`.
    pub register_plugin_id: Option<unsafe extern "C" fn(*const c_char)>,
    /// `ts3plugin_commandKeyword`.
    pub command_keyword: Option<TextExport>,
    /// `ts3plugin_processCommand`.
    pub process_command: Option<unsafe extern "C" fn(u64, *const c_char) -> c_int>,
    /// `ts3plugin_currentServerConnectionChanged`.
    pub current_server_connection_changed: Option<unsafe extern "C" fn(u64)>,
    /// `ts3plugin_infoTitle`.
    pub info_title: Option<TextExport>,
    /// `ts3plugin_infoData`.
    pub info_data: Option<unsafe extern "C" fn(u64, u64, c_int, *mut *mut c_char)>,
    /// `ts3plugin_freeMemory`.
    pub free_memory: Option<unsafe extern "C" fn(*mut c_void)>,
    /// `ts3plugin_requestAutoload`.
    pub request_autoload: Option<NumberExport>,
    /// `ts3plugin_onConnectStatusChangeEvent`.
    pub on_connect_status_change_event: Option<unsafe extern "C" fn(u64, c_int, c_uint)>,
    /// `ts3plugin_onNewChannelEvent`.
    pub on_new_channel_event: Option<unsafe extern "C" fn(u64, u64, u64)>,
    /// `ts3plugin_onNewChannelCreatedEvent`.
    pub on_new_channel_created_event: Option<InvokedChannelExport>,
    /// `ts3plugin_onDelChannelEvent`.
    pub on_del_channel_event: Option<InvokedChannelUpdateExport>,
    /// `ts3plugin_onChannelMoveEvent`.
    pub on_channel_move_event: Option<InvokedChannelExport>,
    /// `ts3plugin_onUpdateChannelEvent`.
    pub on_update_channel_event: Option<unsafe extern "C" fn(u64, u64)>,
    /// `ts3plugin_onUpdateChannelEditedEvent`.
    pub on_update_channel_edited_event: Option<InvokedChannelUpdateExport>,
    /// `ts3plugin_onUpdateClientEvent`: server, client, invoker ID, name, unique ID.
    pub on_update_client_event:
        Option<unsafe extern "C" fn(u64, u16, u16, *const c_char, *const c_char)>,
    /// `ts3plugin_onClientMoveEvent`.
    pub on_client_move_event: Option<ClientMoveExport>,
    /// `ts3plugin_onClientMoveTimeoutEvent`.
    pub on_client_move_timeout_event: Option<ClientMoveExport>,
    /// `ts3plugin_onClientMoveMovedEvent`.
    pub on_client_move_moved_event: Option<InvokedClientMoveExport>,
    /// `ts3plugin_onClientKickFromChannelEvent`.
    pub on_client_kick_from_channel_event: Option<InvokedClientMoveExport>,
    /// `ts3plugin_onClientKickFromServerEvent`.
    pub on_client_kick_from_server_event: Option<InvokedClientMoveExport>,
    /// `ts3plugin_onClientBanFromServerEvent`: a kick plus the ban time before the message.
    #[allow(clippy::type_complexity, reason = "the SDK callback signature")]
    pub on_client_ban_from_server_event: Option<
        unsafe extern "C" fn(
            u64,
            u16,
            u64,
            u64,
            c_int,
            u16,
            *const c_char,
            *const c_char,
            u64,
            *const c_char,
        ),
    >,
    /// `ts3plugin_onServerErrorEvent`: server, message, error, return code, extra message.
    #[allow(clippy::type_complexity, reason = "the SDK callback signature")]
    pub on_server_error_event: Option<
        unsafe extern "C" fn(u64, *const c_char, c_uint, *const c_char, *const c_char) -> c_int,
    >,
    /// `ts3plugin_onServerPermissionErrorEvent`: server, message, error, return code, permission.
    pub on_server_permission_error_event:
        Option<unsafe extern "C" fn(u64, *const c_char, c_uint, *const c_char, c_uint) -> c_int>,
    /// `ts3plugin_onClientSelfVariableUpdateEvent`: server, flag, old value, new value.
    pub on_client_self_variable_update_event:
        Option<unsafe extern "C" fn(u64, c_int, *const c_char, *const c_char)>,
    /// `ts3plugin_onPluginCommandEvent`: server, plugin name, command, invoker ID, name, unique ID.
    pub on_plugin_command_event: Option<
        unsafe extern "C" fn(u64, *const c_char, *const c_char, u16, *const c_char, *const c_char),
    >,
    /// `ts3plugin_onEditPostProcessVoiceDataEvent`: server, client, samples, frames, channels,
    /// speaker array, fill mask.
    pub on_edit_post_process_voice_data_event:
        Option<unsafe extern "C" fn(u64, u16, *mut i16, c_int, c_int, *const c_uint, *mut c_uint)>,
    /// `ts3plugin_onEditMixedPlaybackVoiceDataEvent`: server, samples, frames, channels, speaker
    /// array, fill mask.
    pub on_edit_mixed_playback_voice_data_event:
        Option<unsafe extern "C" fn(u64, *mut i16, c_int, c_int, *const c_uint, *mut c_uint)>,
    /// `ts3plugin_onEditCapturedVoiceDataEvent`: server, samples, frames, channels, edited.
    pub on_edit_captured_voice_data_event:
        Option<unsafe extern "C" fn(u64, *mut i16, c_int, c_int, *mut c_int)>,
    /// `ts3plugin_onCustom3dRolloffCalculationClientEvent`: server, client, distance, volume.
    pub on_custom_3d_rolloff_calculation_client_event:
        Option<unsafe extern "C" fn(u64, u16, f32, *mut f32)>,
}

/// The symbol `name` of `library` as a `Symbol` of type `Export`, or why it is missing.
///
/// # Safety
///
/// `Export` is the exact function pointer type of the symbol's C definition.
unsafe fn symbol<Export: Copy>(
    library: &Library,
    name: &'static str,
) -> Result<Export, libloading::Error> {
    // SAFETY: the caller names the export's true type.
    unsafe { library.get::<Export>(name) }.map(|symbol| *symbol)
}

/// Resolves every export of `library`; the first missing required one is the error, with its
/// name.
///
/// # Safety
///
/// `library` is a TeamSpeak plugin whose `ts3plugin_*` symbols have the SDK's API 26 signatures.
pub(crate) unsafe fn resolve_exports(
    library: &Library,
) -> Result<PluginExports, (&'static str, libloading::Error)> {
    macro_rules! required {
        ($name:literal) => {
            // SAFETY: the caller guarantees the SDK signature of every `ts3plugin_*` symbol, and
            // the field type spells it.
            unsafe { symbol(library, $name) }.map_err(|error| ($name, error))?
        };
    }
    macro_rules! optional {
        ($name:literal) => {
            // SAFETY: as for the required exports.
            unsafe { symbol(library, $name) }.ok()
        };
    }
    Ok(PluginExports {
        name: required!("ts3plugin_name"),
        version: required!("ts3plugin_version"),
        api_version: required!("ts3plugin_apiVersion"),
        author: required!("ts3plugin_author"),
        description: required!("ts3plugin_description"),
        set_function_pointers: required!("ts3plugin_setFunctionPointers"),
        init: required!("ts3plugin_init"),
        shutdown: required!("ts3plugin_shutdown"),
        offers_configure: optional!("ts3plugin_offersConfigure"),
        register_plugin_id: optional!("ts3plugin_registerPluginID"),
        command_keyword: optional!("ts3plugin_commandKeyword"),
        process_command: optional!("ts3plugin_processCommand"),
        current_server_connection_changed: optional!("ts3plugin_currentServerConnectionChanged"),
        info_title: optional!("ts3plugin_infoTitle"),
        info_data: optional!("ts3plugin_infoData"),
        free_memory: optional!("ts3plugin_freeMemory"),
        request_autoload: optional!("ts3plugin_requestAutoload"),
        on_connect_status_change_event: optional!("ts3plugin_onConnectStatusChangeEvent"),
        on_new_channel_event: optional!("ts3plugin_onNewChannelEvent"),
        on_new_channel_created_event: optional!("ts3plugin_onNewChannelCreatedEvent"),
        on_del_channel_event: optional!("ts3plugin_onDelChannelEvent"),
        on_channel_move_event: optional!("ts3plugin_onChannelMoveEvent"),
        on_update_channel_event: optional!("ts3plugin_onUpdateChannelEvent"),
        on_update_channel_edited_event: optional!("ts3plugin_onUpdateChannelEditedEvent"),
        on_update_client_event: optional!("ts3plugin_onUpdateClientEvent"),
        on_client_move_event: optional!("ts3plugin_onClientMoveEvent"),
        on_client_move_timeout_event: optional!("ts3plugin_onClientMoveTimeoutEvent"),
        on_client_move_moved_event: optional!("ts3plugin_onClientMoveMovedEvent"),
        on_client_kick_from_channel_event: optional!("ts3plugin_onClientKickFromChannelEvent"),
        on_client_kick_from_server_event: optional!("ts3plugin_onClientKickFromServerEvent"),
        on_client_ban_from_server_event: optional!("ts3plugin_onClientBanFromServerEvent"),
        on_server_error_event: optional!("ts3plugin_onServerErrorEvent"),
        on_server_permission_error_event: optional!("ts3plugin_onServerPermissionErrorEvent"),
        on_client_self_variable_update_event: optional!(
            "ts3plugin_onClientSelfVariableUpdateEvent"
        ),
        on_plugin_command_event: optional!("ts3plugin_onPluginCommandEvent"),
        on_edit_post_process_voice_data_event: optional!(
            "ts3plugin_onEditPostProcessVoiceDataEvent"
        ),
        on_edit_mixed_playback_voice_data_event: optional!(
            "ts3plugin_onEditMixedPlaybackVoiceDataEvent"
        ),
        on_edit_captured_voice_data_event: optional!("ts3plugin_onEditCapturedVoiceDataEvent"),
        on_custom_3d_rolloff_calculation_client_event: optional!(
            "ts3plugin_onCustom3dRolloffCalculationClientEvent"
        ),
    })
}
