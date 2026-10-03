//! The event half of [`ts3_plugin!`](crate::ts3_plugin): one `extern "C"` shell per TeamSpeak
//! event TFAR handles, forwarding to `$crate::export_glue`.
//!
//! **Role:** the connection, channel, client, error, plugin-command and talk-status exports, with
//! the argument lists of `reference/tfar_1_0/ts/src/plugin.h` at API 26; the talk-status export,
//! which neither original declares, has the SDK `plugin.h` list
//! `(uint64 serverConnectionHandlerID, int status, int isReceivedWhisper, anyID clientID)`.
//!
//! **Position:** expanded only by [`ts3_plugin!`](crate::ts3_plugin); hidden from the docs.
//!
//! **Signals and state:** TeamSpeak raises all of these on its main thread; no state.
//!
//! **Invariants:** the raw arguments reach the glue unchanged; every `unsafe` call relies only on
//! the SDK's promise that string arguments are null or NUL-terminated for the call.

/// Expands to the event exports of [`ts3_plugin!`](crate::ts3_plugin); not for direct use.
#[doc(hidden)]
#[macro_export]
macro_rules! __ts3_plugin_events {
    ($plugin:path) => {
        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onConnectStatusChangeEvent(
            server: u64,
            new_status: ::std::ffi::c_int,
            error_number: ::std::ffi::c_uint,
        ) {
            $crate::export_glue::connect_status_change(&$plugin, server, new_status, error_number);
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onNewChannelEvent(server: u64, channel: u64, parent: u64) {
            $crate::export_glue::new_channel(&$plugin, server, channel, parent);
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onNewChannelCreatedEvent(
            server: u64,
            channel: u64,
            parent: u64,
            invoker_id: u16,
            invoker_name: *const ::std::ffi::c_char,
            invoker_unique_identifier: *const ::std::ffi::c_char,
        ) {
            let invoker = $crate::export_glue::RawInvoker {
                id: invoker_id,
                name: invoker_name,
                unique_identifier: invoker_unique_identifier,
            };
            // SAFETY: TeamSpeak passes null or NUL-terminated strings valid for this call.
            unsafe {
                $crate::export_glue::new_channel_created(&$plugin, server, channel, parent, invoker)
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onDelChannelEvent(
            server: u64,
            channel: u64,
            invoker_id: u16,
            invoker_name: *const ::std::ffi::c_char,
            invoker_unique_identifier: *const ::std::ffi::c_char,
        ) {
            let invoker = $crate::export_glue::RawInvoker {
                id: invoker_id,
                name: invoker_name,
                unique_identifier: invoker_unique_identifier,
            };
            // SAFETY: TeamSpeak passes null or NUL-terminated strings valid for this call.
            unsafe { $crate::export_glue::delete_channel(&$plugin, server, channel, invoker) }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onChannelMoveEvent(
            server: u64,
            channel: u64,
            new_parent: u64,
            invoker_id: u16,
            invoker_name: *const ::std::ffi::c_char,
            invoker_unique_identifier: *const ::std::ffi::c_char,
        ) {
            let invoker = $crate::export_glue::RawInvoker {
                id: invoker_id,
                name: invoker_name,
                unique_identifier: invoker_unique_identifier,
            };
            // SAFETY: TeamSpeak passes null or NUL-terminated strings valid for this call.
            unsafe {
                $crate::export_glue::channel_move(&$plugin, server, channel, new_parent, invoker)
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onUpdateChannelEvent(server: u64, channel: u64) {
            $crate::export_glue::update_channel(&$plugin, server, channel);
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onUpdateChannelEditedEvent(
            server: u64,
            channel: u64,
            invoker_id: u16,
            invoker_name: *const ::std::ffi::c_char,
            invoker_unique_identifier: *const ::std::ffi::c_char,
        ) {
            let invoker = $crate::export_glue::RawInvoker {
                id: invoker_id,
                name: invoker_name,
                unique_identifier: invoker_unique_identifier,
            };
            // SAFETY: TeamSpeak passes null or NUL-terminated strings valid for this call.
            unsafe {
                $crate::export_glue::update_channel_edited(&$plugin, server, channel, invoker)
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onUpdateClientEvent(
            server: u64,
            client: u16,
            invoker_id: u16,
            invoker_name: *const ::std::ffi::c_char,
            invoker_unique_identifier: *const ::std::ffi::c_char,
        ) {
            let invoker = $crate::export_glue::RawInvoker {
                id: invoker_id,
                name: invoker_name,
                unique_identifier: invoker_unique_identifier,
            };
            // SAFETY: TeamSpeak passes null or NUL-terminated strings valid for this call.
            unsafe { $crate::export_glue::update_client(&$plugin, server, client, invoker) }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onClientMoveEvent(
            server: u64,
            client: u16,
            old_channel: u64,
            new_channel: u64,
            visibility: ::std::ffi::c_int,
            move_message: *const ::std::ffi::c_char,
        ) {
            let movement = $crate::export_glue::RawClientMove {
                server,
                client,
                old_channel,
                new_channel,
                visibility,
                message: move_message,
            };
            let kind = $crate::export_glue::PlainMoveKind::Move;
            // SAFETY: TeamSpeak passes a null or NUL-terminated message valid for this call.
            unsafe { $crate::export_glue::plain_client_move(&$plugin, kind, movement) }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onClientMoveTimeoutEvent(
            server: u64,
            client: u16,
            old_channel: u64,
            new_channel: u64,
            visibility: ::std::ffi::c_int,
            timeout_message: *const ::std::ffi::c_char,
        ) {
            let movement = $crate::export_glue::RawClientMove {
                server,
                client,
                old_channel,
                new_channel,
                visibility,
                message: timeout_message,
            };
            let kind = $crate::export_glue::PlainMoveKind::Timeout;
            // SAFETY: TeamSpeak passes a null or NUL-terminated message valid for this call.
            unsafe { $crate::export_glue::plain_client_move(&$plugin, kind, movement) }
        }

        $crate::__ts3_plugin_invoked_move!(
            $plugin,
            ts3plugin_onClientMoveMovedEvent,
            $crate::export_glue::InvokedMoveKind::Moved
        );
        $crate::__ts3_plugin_invoked_move!(
            $plugin,
            ts3plugin_onClientKickFromChannelEvent,
            $crate::export_glue::InvokedMoveKind::KickFromChannel
        );
        $crate::__ts3_plugin_invoked_move!(
            $plugin,
            ts3plugin_onClientKickFromServerEvent,
            $crate::export_glue::InvokedMoveKind::KickFromServer
        );

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        #[allow(clippy::too_many_arguments, reason = "the SDK callback signature")]
        extern "C" fn ts3plugin_onClientBanFromServerEvent(
            server: u64,
            client: u16,
            old_channel: u64,
            new_channel: u64,
            visibility: ::std::ffi::c_int,
            kicker_id: u16,
            kicker_name: *const ::std::ffi::c_char,
            kicker_unique_identifier: *const ::std::ffi::c_char,
            time: u64,
            kick_message: *const ::std::ffi::c_char,
        ) {
            let movement = $crate::export_glue::RawClientMove {
                server,
                client,
                old_channel,
                new_channel,
                visibility,
                message: kick_message,
            };
            let kicker = $crate::export_glue::RawInvoker {
                id: kicker_id,
                name: kicker_name,
                unique_identifier: kicker_unique_identifier,
            };
            let kind = $crate::export_glue::InvokedMoveKind::BanFromServer(time);
            // SAFETY: TeamSpeak passes null or NUL-terminated strings valid for this call.
            unsafe { $crate::export_glue::invoked_client_move(&$plugin, kind, movement, kicker) }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onServerErrorEvent(
            server: u64,
            error_message: *const ::std::ffi::c_char,
            error: ::std::ffi::c_uint,
            return_code: *const ::std::ffi::c_char,
            extra_message: *const ::std::ffi::c_char,
        ) -> ::std::ffi::c_int {
            // SAFETY: TeamSpeak passes null or NUL-terminated strings valid for this call.
            unsafe {
                $crate::export_glue::server_error(
                    &$plugin,
                    server,
                    error_message,
                    error,
                    return_code,
                    extra_message,
                )
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onServerPermissionErrorEvent(
            server: u64,
            error_message: *const ::std::ffi::c_char,
            error: ::std::ffi::c_uint,
            return_code: *const ::std::ffi::c_char,
            failed_permission_id: ::std::ffi::c_uint,
        ) -> ::std::ffi::c_int {
            // SAFETY: TeamSpeak passes null or NUL-terminated strings valid for this call.
            unsafe {
                $crate::export_glue::server_permission_error(
                    &$plugin,
                    server,
                    error_message,
                    error,
                    return_code,
                    failed_permission_id,
                )
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onClientSelfVariableUpdateEvent(
            server: u64,
            flag: ::std::ffi::c_int,
            old_value: *const ::std::ffi::c_char,
            new_value: *const ::std::ffi::c_char,
        ) {
            // SAFETY: TeamSpeak passes null or NUL-terminated strings valid for this call.
            unsafe {
                $crate::export_glue::client_self_variable_update(
                    &$plugin, server, flag, old_value, new_value,
                )
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onTalkStatusChangeEvent(
            server: u64,
            status: ::std::ffi::c_int,
            is_received_whisper: ::std::ffi::c_int,
            client: u16,
        ) {
            $crate::export_glue::talk_status_change(
                &$plugin,
                server,
                status,
                is_received_whisper,
                client,
            );
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onPluginCommandEvent(
            server: u64,
            plugin_name: *const ::std::ffi::c_char,
            plugin_command: *const ::std::ffi::c_char,
            invoker_client_id: u16,
            invoker_name: *const ::std::ffi::c_char,
            invoker_unique_identity: *const ::std::ffi::c_char,
        ) {
            let invoker = $crate::export_glue::RawInvoker {
                id: invoker_client_id,
                name: invoker_name,
                unique_identifier: invoker_unique_identity,
            };
            // SAFETY: TeamSpeak passes null or NUL-terminated strings valid for this call.
            unsafe {
                $crate::export_glue::plugin_command(
                    &$plugin,
                    server,
                    plugin_name,
                    plugin_command,
                    invoker,
                )
            }
        }
    };
}

/// Expands to one move-with-invoker export (moved, kicked from channel, kicked from server); not
/// for direct use.
#[doc(hidden)]
#[macro_export]
macro_rules! __ts3_plugin_invoked_move {
    ($plugin:path, $export:ident, $kind:expr) => {
        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        #[allow(clippy::too_many_arguments, reason = "the SDK callback signature")]
        extern "C" fn $export(
            server: u64,
            client: u16,
            old_channel: u64,
            new_channel: u64,
            visibility: ::std::ffi::c_int,
            invoker_id: u16,
            invoker_name: *const ::std::ffi::c_char,
            invoker_unique_identifier: *const ::std::ffi::c_char,
            message: *const ::std::ffi::c_char,
        ) {
            let movement = $crate::export_glue::RawClientMove {
                server,
                client,
                old_channel,
                new_channel,
                visibility,
                message,
            };
            let invoker = $crate::export_glue::RawInvoker {
                id: invoker_id,
                name: invoker_name,
                unique_identifier: invoker_unique_identifier,
            };
            // SAFETY: TeamSpeak passes null or NUL-terminated strings valid for this call.
            unsafe { $crate::export_glue::invoked_client_move(&$plugin, $kind, movement, invoker) }
        }
    };
}
