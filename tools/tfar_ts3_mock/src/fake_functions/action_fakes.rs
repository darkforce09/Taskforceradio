//! The fakes for the requests that change something: client moves, local mutes, plugin
//! commands, 3D sound and the capture pre-processor.
//!
//! **Role:** `requestClientMove`, `requestMuteClients`, `requestUnmuteClients`,
//! `sendPluginCommand`, `systemset3DSettings`, `systemset3DListenerAttributes`,
//! `channelset3DAttributes`, `getPreProcessorConfigValue` and `setPreProcessorConfigValue`.
//!
//! **Position:** installed into the table by `fake_table`; the plugin calls them.
//!
//! **Signals and state:** each takes the fake-state lock once; called from any plugin thread.
//!
//! **Invariants:** a request is logged with its arguments copied; mutes change the clients'
//! `is_muted` flag; a move is only logged (the server's answer is a test's
//! [`MockHost::client_moves`](crate::MockHost::client_moves)); pre-processor values live in the
//! fake state, an unknown one answering `ERROR_parameter_invalid`.

use std::ffi::{c_char, c_int, c_uint};

use tfar_ts3_host::{ChannelId, ClientId, ServerConnectionHandlerId};
use ts3plugin_sys::public_definitions as sdk;

use super::fake_answers::{
    ERROR_NOT_CONNECTED, ERROR_OK, ERROR_PARAMETER_INVALID, copied_text, hand_out_string,
    read_vector, terminated_clients,
};
use crate::fake_state::host_session::with_fake_teamspeak;
use crate::{HostCall, HostQuery, SentCommandTarget};

/// `requestClientMove`: logged.
pub(crate) extern "C" fn request_client_move(
    server_connection_handler_id: u64,
    client_id: u16,
    new_channel_id: u64,
    password: *const c_char,
    _return_code: *const c_char,
) -> c_uint {
    let call = HostCall::RequestClientMove {
        server: ServerConnectionHandlerId(server_connection_handler_id),
        client: ClientId(client_id),
        channel: ChannelId(new_channel_id),
        password: copied_text(password),
    };
    with_fake_teamspeak(|fake| fake.record(call));
    ERROR_OK
}

/// Sets the local mute of `clients` on `server`, ignoring clients the server does not have.
fn set_local_mute(
    server: ServerConnectionHandlerId,
    clients: &[ClientId],
    is_muted: bool,
    call: HostCall,
) -> c_uint {
    with_fake_teamspeak(|fake| {
        fake.record(call);
        let Some(fake_server) = fake.server_mut(server) else {
            return ERROR_NOT_CONNECTED;
        };
        for client in clients {
            if let Some(fake_client) = fake_server.clients.get_mut(client) {
                fake_client.is_muted = is_muted;
            }
        }
        ERROR_OK
    })
}

/// `requestMuteClients`: mutes the listed clients locally.
pub(crate) extern "C" fn request_mute_clients(
    server_connection_handler_id: u64,
    client_id_array: *const u16,
    _return_code: *const c_char,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    let clients = terminated_clients(client_id_array);
    let call = HostCall::RequestMuteClients {
        server,
        clients: clients.clone(),
    };
    set_local_mute(server, &clients, true, call)
}

/// `requestUnmuteClients`: unmutes the listed clients locally.
pub(crate) extern "C" fn request_unmute_clients(
    server_connection_handler_id: u64,
    client_id_array: *const u16,
    _return_code: *const c_char,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    let clients = terminated_clients(client_id_array);
    let call = HostCall::RequestUnmuteClients {
        server,
        clients: clients.clone(),
    };
    set_local_mute(server, &clients, false, call)
}

/// `sendPluginCommand`: logged with its receivers.
pub(crate) extern "C" fn send_plugin_command(
    server_connection_handler_id: u64,
    plugin_id: *const c_char,
    command: *const c_char,
    target_mode: c_int,
    target_ids: *const u16,
    _return_code: *const c_char,
) {
    let target = match target_mode {
        mode if mode == sdk::PluginTargetMode::CurrentChannel as c_int => {
            SentCommandTarget::CurrentChannel
        }
        mode if mode == sdk::PluginTargetMode::Server as c_int => SentCommandTarget::Server,
        mode if mode == sdk::PluginTargetMode::Client as c_int => {
            SentCommandTarget::Clients(terminated_clients(target_ids))
        }
        mode if mode == sdk::PluginTargetMode::CurrentChannelSubscribedClients as c_int => {
            SentCommandTarget::CurrentChannelSubscribedClients
        }
        other => SentCommandTarget::Other(other),
    };
    let call = HostCall::SendPluginCommand {
        server: ServerConnectionHandlerId(server_connection_handler_id),
        plugin_id: copied_text(plugin_id),
        command: copied_text(command),
        target,
    };
    with_fake_teamspeak(|fake| fake.record(call));
}

/// `systemset3DSettings`: logged.
pub(crate) extern "C" fn systemset3d_settings(
    server_connection_handler_id: u64,
    distance_factor: f32,
    rolloff_scale: f32,
) -> c_uint {
    let call = HostCall::Systemset3DSettings {
        server: ServerConnectionHandlerId(server_connection_handler_id),
        distance_factor,
        rolloff_scale,
    };
    with_fake_teamspeak(|fake| fake.record(call));
    ERROR_OK
}

/// `systemset3DListenerAttributes`: logged.
pub(crate) extern "C" fn systemset3d_listener_attributes(
    server_connection_handler_id: u64,
    position: *const sdk::Ts3Vector,
    forward: *const sdk::Ts3Vector,
    up: *const sdk::Ts3Vector,
) -> c_uint {
    let call = HostCall::Systemset3DListenerAttributes {
        server: ServerConnectionHandlerId(server_connection_handler_id),
        position: read_vector(position),
        forward: read_vector(forward),
        up: read_vector(up),
    };
    with_fake_teamspeak(|fake| fake.record(call));
    ERROR_OK
}

/// `channelset3DAttributes`: logged.
pub(crate) extern "C" fn channelset3d_attributes(
    server_connection_handler_id: u64,
    client_id: u16,
    position: *const sdk::Ts3Vector,
) -> c_uint {
    let call = HostCall::Channelset3DAttributes {
        server: ServerConnectionHandlerId(server_connection_handler_id),
        client: ClientId(client_id),
        position: read_vector(position),
    };
    with_fake_teamspeak(|fake| fake.record(call));
    ERROR_OK
}

/// `getPreProcessorConfigValue`: a `malloc`ed copy of the stored value.
pub(crate) extern "C" fn get_pre_processor_config_value(
    server_connection_handler_id: u64,
    ident: *const c_char,
    result: *mut *mut c_char,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    let ident = copied_text(ident);
    with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::PreProcessorConfigValue(
            server,
            ident.clone(),
        )));
        let Some(value) = fake.preprocessor_values.get(&ident).cloned() else {
            return ERROR_PARAMETER_INVALID;
        };
        hand_out_string(
            &mut fake.ledger,
            "getPreProcessorConfigValue",
            &value,
            result,
        )
    })
}

/// `setPreProcessorConfigValue`: stores the value and logs the call.
pub(crate) extern "C" fn set_pre_processor_config_value(
    server_connection_handler_id: u64,
    ident: *const c_char,
    value: *const c_char,
) -> c_uint {
    let ident = copied_text(ident);
    let value = copied_text(value);
    with_fake_teamspeak(|fake| {
        fake.record(HostCall::SetPreProcessorConfigValue {
            server: ServerConnectionHandlerId(server_connection_handler_id),
            ident: ident.clone(),
            value: value.clone(),
        });
        fake.preprocessor_values.insert(ident, value);
        ERROR_OK
    })
}
