//! The fakes for client, own-client, channel and server variables.
//!
//! **Role:** `getClientVariableAsString`, `getClientVariableAsInt`, `getClientSelfVariableAsInt`,
//! `setClientSelfVariableAsString`, `setClientSelfVariableAsInt`, `flushClientSelfUpdates`,
//! `getChannelVariableAsString` and `getServerVariableAsString`, over the properties TFAR reads
//! and writes.
//!
//! **Position:** installed into the table by `fake_table`; the plugin calls them.
//!
//! **Signals and state:** each takes the fake-state lock once; called from any plugin thread.
//!
//! **Invariants:** a set changes the own client at once (a flush only confirms it); a flag the
//! fake does not model answers `ERROR_parameter_invalid`; strings are `malloc`ed and recorded in
//! the ledger; every call is logged.

use std::ffi::{c_char, c_int, c_uint};

use tfar_ts3_host::{ChannelId, ClientId, ServerConnectionHandlerId};
use ts3plugin_sys::public_definitions as sdk;

use super::fake_answers::{
    ERROR_CHANNEL_INVALID_ID, ERROR_CLIENT_INVALID_ID, ERROR_NOT_CONNECTED, ERROR_OK,
    ERROR_PARAMETER_INVALID, copied_text, hand_out_string, write_answer,
};
use crate::fake_state::host_session::with_fake_teamspeak;
use crate::{FakeClient, HostCall, HostQuery};

/// `CLIENT_UNIQUE_IDENTIFIER`.
const CLIENT_UNIQUE_IDENTIFIER: usize = sdk::ClientProperties::UniqueIdentifier as usize;
/// `CLIENT_NICKNAME`.
const CLIENT_NICKNAME: usize = sdk::ClientProperties::Nickname as usize;
/// `CLIENT_FLAG_TALKING`.
const CLIENT_FLAG_TALKING: usize = sdk::ClientProperties::FlagTalking as usize;
/// `CLIENT_INPUT_MUTED`.
const CLIENT_INPUT_MUTED: usize = sdk::ClientProperties::InputMuted as usize;
/// `CLIENT_OUTPUT_MUTED`.
const CLIENT_OUTPUT_MUTED: usize = sdk::ClientProperties::OutputMuted as usize;
/// `CLIENT_INPUT_DEACTIVATED`.
const CLIENT_INPUT_DEACTIVATED: usize = sdk::ClientProperties::InputDeactivated as usize;
/// `CLIENT_META_DATA`.
const CLIENT_META_DATA: usize = sdk::ClientProperties::MetaData as usize;
/// `CLIENT_IS_MUTED`.
const CLIENT_IS_MUTED: usize = sdk::ClientProperties::IsMuted as usize;
/// `CHANNEL_NAME`.
const CHANNEL_NAME: usize = sdk::ChannelProperties::Name as usize;
/// `VIRTUALSERVER_UNIQUE_IDENTIFIER`.
const SERVER_UNIQUE_IDENTIFIER: usize = sdk::VirtualServerProperties::UniqueIdentifier as usize;
/// `VIRTUALSERVER_NAME`.
const SERVER_NAME: usize = sdk::VirtualServerProperties::Name as usize;

/// The string property `flag` of `client`, if the fake models it.
fn client_string(client: &FakeClient, flag: usize) -> Option<String> {
    match flag {
        CLIENT_UNIQUE_IDENTIFIER => Some(client.unique_identifier.clone()),
        CLIENT_NICKNAME => Some(client.nickname.clone()),
        CLIENT_META_DATA => Some(client.metadata.clone()),
        _ => None,
    }
}

/// The integer property `flag` of `client`, if the fake models it.
fn client_int(client: &FakeClient, flag: usize) -> Option<i32> {
    match flag {
        CLIENT_FLAG_TALKING => Some(i32::from(client.is_talking)),
        CLIENT_INPUT_MUTED => Some(client.input_muted),
        CLIENT_OUTPUT_MUTED => Some(client.output_muted),
        CLIENT_INPUT_DEACTIVATED => Some(client.input_deactivated),
        CLIENT_IS_MUTED => Some(i32::from(client.is_muted)),
        _ => None,
    }
}

/// `getClientVariableAsString`: nickname, unique identifier or metadata.
pub(crate) extern "C" fn get_client_variable_as_string(
    server_connection_handler_id: u64,
    client_id: u16,
    flag: usize,
    result: *mut *mut c_char,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    let client = ClientId(client_id);
    with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::ClientVariableAsString(
            server, client, flag,
        )));
        let text = match fake
            .server(server)
            .map(|fake_server| fake_server.clients.get(&client))
        {
            None => return ERROR_NOT_CONNECTED,
            Some(None) => return ERROR_CLIENT_INVALID_ID,
            Some(Some(fake_client)) => client_string(fake_client, flag),
        };
        let Some(text) = text else {
            return ERROR_PARAMETER_INVALID;
        };
        hand_out_string(&mut fake.ledger, "getClientVariableAsString", &text, result)
    })
}

/// `getClientVariableAsInt`: talking, mute and deactivation flags.
pub(crate) extern "C" fn get_client_variable_as_int(
    server_connection_handler_id: u64,
    client_id: u16,
    flag: usize,
    result: *mut c_int,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    let client = ClientId(client_id);
    let value = with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::ClientVariableAsInt(
            server, client, flag,
        )));
        let fake_server = fake.server(server).ok_or(ERROR_NOT_CONNECTED)?;
        let fake_client = fake_server
            .clients
            .get(&client)
            .ok_or(ERROR_CLIENT_INVALID_ID)?;
        client_int(fake_client, flag).ok_or(ERROR_PARAMETER_INVALID)
    });
    match value {
        Ok(value) => write_answer(result, value),
        Err(code) => code,
    }
}

/// `getClientSelfVariableAsInt`: an integer property of our own client.
pub(crate) extern "C" fn get_client_self_variable_as_int(
    server_connection_handler_id: u64,
    flag: usize,
    result: *mut c_int,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    let value = with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::ClientSelfVariableAsInt(
            server, flag,
        )));
        let own = fake.own_client_mut(server).ok_or(ERROR_NOT_CONNECTED)?;
        client_int(own, flag).ok_or(ERROR_PARAMETER_INVALID)
    });
    match value {
        Ok(value) => write_answer(result, value),
        Err(code) => code,
    }
}

/// `setClientSelfVariableAsString`: our nickname or metadata.
pub(crate) extern "C" fn set_client_self_variable_as_string(
    server_connection_handler_id: u64,
    flag: usize,
    value: *const c_char,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    let value = copied_text(value);
    with_fake_teamspeak(|fake| {
        fake.record(HostCall::SetClientSelfVariableAsString {
            server,
            flag,
            value: value.clone(),
        });
        let Some(own) = fake.own_client_mut(server) else {
            return ERROR_NOT_CONNECTED;
        };
        match flag {
            CLIENT_NICKNAME => own.nickname = value,
            CLIENT_META_DATA => own.metadata = value,
            _ => return ERROR_PARAMETER_INVALID,
        }
        ERROR_OK
    })
}

/// `setClientSelfVariableAsInt`: our input mute, output mute or input deactivation.
pub(crate) extern "C" fn set_client_self_variable_as_int(
    server_connection_handler_id: u64,
    flag: usize,
    value: c_int,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    with_fake_teamspeak(|fake| {
        fake.record(HostCall::SetClientSelfVariableAsInt {
            server,
            flag,
            value,
        });
        let Some(own) = fake.own_client_mut(server) else {
            return ERROR_NOT_CONNECTED;
        };
        match flag {
            CLIENT_INPUT_MUTED => own.input_muted = value,
            CLIENT_OUTPUT_MUTED => own.output_muted = value,
            CLIENT_INPUT_DEACTIVATED => own.input_deactivated = value,
            _ => return ERROR_PARAMETER_INVALID,
        }
        ERROR_OK
    })
}

/// `flushClientSelfUpdates`: logged; the sets already took effect.
pub(crate) extern "C" fn flush_client_self_updates(
    server_connection_handler_id: u64,
    _return_code: *const c_char,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    with_fake_teamspeak(|fake| {
        fake.record(HostCall::FlushClientSelfUpdates { server });
        if fake.server(server).is_some() {
            ERROR_OK
        } else {
            ERROR_NOT_CONNECTED
        }
    })
}

/// `getChannelVariableAsString`: a channel's name.
pub(crate) extern "C" fn get_channel_variable_as_string(
    server_connection_handler_id: u64,
    channel_id: u64,
    flag: usize,
    result: *mut *mut c_char,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    let channel = ChannelId(channel_id);
    with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::ChannelVariableAsString(
            server, channel, flag,
        )));
        let name = match fake
            .server(server)
            .map(|fake_server| fake_server.channels.get(&channel))
        {
            None => return ERROR_NOT_CONNECTED,
            Some(None) => return ERROR_CHANNEL_INVALID_ID,
            Some(Some(fake_channel)) => fake_channel.name.clone(),
        };
        if flag != CHANNEL_NAME {
            return ERROR_PARAMETER_INVALID;
        }
        hand_out_string(
            &mut fake.ledger,
            "getChannelVariableAsString",
            &name,
            result,
        )
    })
}

/// `getServerVariableAsString`: the server's name or unique identifier.
pub(crate) extern "C" fn get_server_variable_as_string(
    server_connection_handler_id: u64,
    flag: usize,
    result: *mut *mut c_char,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::ServerVariableAsString(
            server, flag,
        )));
        let Some(fake_server) = fake.server(server) else {
            return ERROR_NOT_CONNECTED;
        };
        let text = match flag {
            SERVER_NAME => fake_server.name.clone(),
            SERVER_UNIQUE_IDENTIFIER => fake_server.unique_identifier.clone(),
            _ => return ERROR_PARAMETER_INVALID,
        };
        hand_out_string(&mut fake.ledger, "getServerVariableAsString", &text, result)
    })
}
