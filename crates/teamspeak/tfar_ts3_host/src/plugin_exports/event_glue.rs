//! The bodies of the event exports: connection, channel and client events, server errors, our own
//! variable updates, plugin commands and talk status changes.
//!
//! **Role:** turns TeamSpeak's raw event arguments into the newtypes, `&str`s and bundles of
//! `event_arguments` and calls the matching [`Ts3Plugin`] method.
//!
//! **Position:** called only from the expansion of [`ts3_plugin!`](crate::ts3_plugin); public but
//! hidden.
//!
//! **Signals and state:** TeamSpeak raises all of these on its main thread; no state here.
//!
//! **Invariants:** every string pointer is null-checked and read lossily for the call only;
//! nothing here panics.

use std::ffi::{c_char, c_int, c_uint};

use super::event_arguments::{
    ClientMove, Invoker, PluginCommand, ServerError, ServerPermissionError,
};
use super::plugin_trait::Ts3Plugin;
use crate::ts3_functions::owned_memory::borrowed_text;
use crate::{
    ChannelId, ClientId, ConnectStatus, ServerConnectionHandlerId, TalkStatus, Visibility,
};

/// The raw "who did it" arguments TeamSpeak passes as a triple.
#[derive(Clone, Copy)]
pub struct RawInvoker {
    /// `anyID invokerID` (or mover, kicker).
    pub id: u16,
    /// `const char* invokerName`.
    pub name: *const c_char,
    /// `const char* invokerUniqueIdentifier`.
    pub unique_identifier: *const c_char,
}

/// The raw arguments the move, timeout, kick and ban events share.
#[derive(Clone, Copy)]
pub struct RawClientMove {
    /// `uint64 serverConnectionHandlerID`.
    pub server: u64,
    /// `anyID clientID`.
    pub client: u16,
    /// `uint64 oldChannelID`.
    pub old_channel: u64,
    /// `uint64 newChannelID`.
    pub new_channel: u64,
    /// `int visibility`.
    pub visibility: c_int,
    /// `const char* moveMessage` (or timeout, kick message).
    pub message: *const c_char,
}

/// Calls `deliver` with the safe form of `raw`.
///
/// # Safety
///
/// Both string pointers are null or NUL-terminated strings valid for this call.
unsafe fn with_invoker(raw: RawInvoker, deliver: impl FnOnce(Invoker<'_>)) {
    // SAFETY: null or NUL-terminated for the call, by the caller's contract.
    let name = unsafe { borrowed_text(raw.name) };
    // SAFETY: as above.
    let unique_identifier = unsafe { borrowed_text(raw.unique_identifier) };
    deliver(Invoker {
        id: ClientId(raw.id),
        name: &name,
        unique_identifier: &unique_identifier,
    });
}

/// Calls `deliver` with the safe form of `raw`.
///
/// # Safety
///
/// `raw.message` is null or a NUL-terminated string valid for this call.
unsafe fn with_client_move(raw: RawClientMove, deliver: impl FnOnce(ClientMove<'_>)) {
    // SAFETY: null or NUL-terminated for the call, by the caller's contract.
    let message = unsafe { borrowed_text(raw.message) };
    deliver(ClientMove {
        server: ServerConnectionHandlerId(raw.server),
        client: ClientId(raw.client),
        old_channel: ChannelId(raw.old_channel),
        new_channel: ChannelId(raw.new_channel),
        visibility: Visibility::from_raw(raw.visibility),
        message: &message,
    });
}

/// `onConnectStatusChangeEvent`.
pub fn connect_status_change<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    status: c_int,
    error: c_uint,
) {
    plugin.on_connect_status_change(
        ServerConnectionHandlerId(server),
        ConnectStatus::from_raw(status),
        error,
    );
}

/// `onNewChannelEvent`.
pub fn new_channel<P: Ts3Plugin>(plugin: &'static P, server: u64, channel: u64, parent: u64) {
    plugin.on_new_channel(
        ServerConnectionHandlerId(server),
        ChannelId(channel),
        ChannelId(parent),
    );
}

/// `onNewChannelCreatedEvent`.
///
/// # Safety
///
/// The invoker's strings are null or NUL-terminated strings valid for this call.
pub unsafe fn new_channel_created<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    channel: u64,
    parent: u64,
    invoker: RawInvoker,
) {
    // SAFETY: the caller's contract covers the invoker strings.
    unsafe {
        with_invoker(invoker, |invoker| {
            plugin.on_new_channel_created(
                ServerConnectionHandlerId(server),
                ChannelId(channel),
                ChannelId(parent),
                invoker,
            );
        });
    }
}

/// `onDelChannelEvent`.
///
/// # Safety
///
/// The invoker's strings are null or NUL-terminated strings valid for this call.
pub unsafe fn delete_channel<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    channel: u64,
    invoker: RawInvoker,
) {
    // SAFETY: the caller's contract covers the invoker strings.
    unsafe {
        with_invoker(invoker, |invoker| {
            plugin.on_delete_channel(
                ServerConnectionHandlerId(server),
                ChannelId(channel),
                invoker,
            );
        });
    }
}

/// `onChannelMoveEvent`.
///
/// # Safety
///
/// The invoker's strings are null or NUL-terminated strings valid for this call.
pub unsafe fn channel_move<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    channel: u64,
    new_parent: u64,
    invoker: RawInvoker,
) {
    // SAFETY: the caller's contract covers the invoker strings.
    unsafe {
        with_invoker(invoker, |invoker| {
            plugin.on_channel_move(
                ServerConnectionHandlerId(server),
                ChannelId(channel),
                ChannelId(new_parent),
                invoker,
            );
        });
    }
}

/// `onUpdateChannelEvent`.
pub fn update_channel<P: Ts3Plugin>(plugin: &'static P, server: u64, channel: u64) {
    plugin.on_update_channel(ServerConnectionHandlerId(server), ChannelId(channel));
}

/// `onUpdateChannelEditedEvent`.
///
/// # Safety
///
/// The invoker's strings are null or NUL-terminated strings valid for this call.
pub unsafe fn update_channel_edited<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    channel: u64,
    invoker: RawInvoker,
) {
    // SAFETY: the caller's contract covers the invoker strings.
    unsafe {
        with_invoker(invoker, |invoker| {
            plugin.on_update_channel_edited(
                ServerConnectionHandlerId(server),
                ChannelId(channel),
                invoker,
            );
        });
    }
}

/// `onUpdateClientEvent`.
///
/// # Safety
///
/// The invoker's strings are null or NUL-terminated strings valid for this call.
pub unsafe fn update_client<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    client: u16,
    invoker: RawInvoker,
) {
    // SAFETY: the caller's contract covers the invoker strings.
    unsafe {
        with_invoker(invoker, |invoker| {
            plugin.on_update_client(ServerConnectionHandlerId(server), ClientId(client), invoker);
        });
    }
}

/// Which of the moves without an invoker TeamSpeak reported.
#[derive(Clone, Copy)]
pub enum PlainMoveKind {
    /// `onClientMoveEvent`.
    Move,
    /// `onClientMoveTimeoutEvent`.
    Timeout,
}

/// `onClientMoveEvent` and `onClientMoveTimeoutEvent`.
///
/// # Safety
///
/// `movement.message` is null or a NUL-terminated string valid for this call.
pub unsafe fn plain_client_move<P: Ts3Plugin>(
    plugin: &'static P,
    kind: PlainMoveKind,
    movement: RawClientMove,
) {
    // SAFETY: the caller's contract covers the message string.
    unsafe {
        with_client_move(movement, |movement| match kind {
            PlainMoveKind::Move => plugin.on_client_move(movement),
            PlainMoveKind::Timeout => plugin.on_client_move_timeout(movement),
        });
    }
}

/// Which of the moves with an invoker TeamSpeak reported.
#[derive(Clone, Copy)]
pub enum InvokedMoveKind {
    /// `onClientMoveMovedEvent`.
    Moved,
    /// `onClientKickFromChannelEvent`.
    KickFromChannel,
    /// `onClientKickFromServerEvent`.
    KickFromServer,
    /// `onClientBanFromServerEvent`, with the ban length in seconds.
    BanFromServer(u64),
}

/// `onClientMoveMovedEvent`, `onClientKickFromChannelEvent`, `onClientKickFromServerEvent` and
/// `onClientBanFromServerEvent`.
///
/// # Safety
///
/// The message and invoker strings are null or NUL-terminated strings valid for this call.
pub unsafe fn invoked_client_move<P: Ts3Plugin>(
    plugin: &'static P,
    kind: InvokedMoveKind,
    movement: RawClientMove,
    invoker: RawInvoker,
) {
    // SAFETY: the caller's contract covers the message and invoker strings.
    unsafe {
        with_client_move(movement, |movement| {
            // SAFETY: as above, for the invoker strings.
            with_invoker(invoker, |invoker| match kind {
                InvokedMoveKind::Moved => plugin.on_client_move_moved(movement, invoker),
                InvokedMoveKind::KickFromChannel => {
                    plugin.on_client_kick_from_channel(movement, invoker);
                }
                InvokedMoveKind::KickFromServer => {
                    plugin.on_client_kick_from_server(movement, invoker);
                }
                InvokedMoveKind::BanFromServer(seconds) => {
                    plugin.on_client_ban_from_server(movement, invoker, seconds);
                }
            });
        });
    }
}

/// `onServerErrorEvent`.
///
/// # Safety
///
/// The three strings are null or NUL-terminated strings valid for this call.
pub unsafe fn server_error<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    message: *const c_char,
    error: c_uint,
    return_code: *const c_char,
    extra_message: *const c_char,
) -> c_int {
    // SAFETY: null or NUL-terminated for the call, by the caller's contract (all three).
    let (message, return_code, extra_message) = unsafe {
        (
            borrowed_text(message),
            borrowed_text(return_code),
            borrowed_text(extra_message),
        )
    };
    plugin
        .on_server_error(ServerError {
            server: ServerConnectionHandlerId(server),
            message: &message,
            error,
            return_code: &return_code,
            extra_message: &extra_message,
        })
        .to_raw()
}

/// `onServerPermissionErrorEvent`.
///
/// # Safety
///
/// The two strings are null or NUL-terminated strings valid for this call.
pub unsafe fn server_permission_error<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    message: *const c_char,
    error: c_uint,
    return_code: *const c_char,
    failed_permission_id: c_uint,
) -> c_int {
    // SAFETY: null or NUL-terminated for the call, by the caller's contract (both).
    let (message, return_code) = unsafe { (borrowed_text(message), borrowed_text(return_code)) };
    plugin
        .on_server_permission_error(ServerPermissionError {
            server: ServerConnectionHandlerId(server),
            message: &message,
            error,
            return_code: &return_code,
            failed_permission_id,
        })
        .to_raw()
}

/// `onClientSelfVariableUpdateEvent`.
///
/// # Safety
///
/// Both values are null or NUL-terminated strings valid for this call.
pub unsafe fn client_self_variable_update<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    flag: c_int,
    old_value: *const c_char,
    new_value: *const c_char,
) {
    // SAFETY: null or NUL-terminated for the call, by the caller's contract (both).
    let (old_value, new_value) = unsafe { (borrowed_text(old_value), borrowed_text(new_value)) };
    plugin.on_client_self_variable_update(
        ServerConnectionHandlerId(server),
        flag,
        &old_value,
        &new_value,
    );
}

/// `onPluginCommandEvent` (API 26: with the invoker's ID, name and unique identity).
///
/// # Safety
///
/// The plugin name, command and invoker strings are null or NUL-terminated strings valid for this
/// call.
pub unsafe fn plugin_command<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    plugin_name: *const c_char,
    command: *const c_char,
    invoker: RawInvoker,
) {
    // SAFETY: null or NUL-terminated for the call, by the caller's contract (both).
    let (plugin_name, command) = unsafe { (borrowed_text(plugin_name), borrowed_text(command)) };
    // SAFETY: the caller's contract covers the invoker strings.
    unsafe {
        with_invoker(invoker, |invoker| {
            plugin.on_plugin_command(PluginCommand {
                server: ServerConnectionHandlerId(server),
                plugin_name: &plugin_name,
                command: &command,
                invoker,
            });
        });
    }
}

/// `onTalkStatusChangeEvent`: `status` is an `enum TalkStatus` value and `is_received_whisper`
/// is non-zero for a whisper.
pub fn talk_status_change<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    status: c_int,
    is_received_whisper: c_int,
    client: u16,
) {
    plugin.on_talk_status_change(
        ServerConnectionHandlerId(server),
        TalkStatus::from_raw(status),
        is_received_whisper != 0,
        ClientId(client),
    );
}
