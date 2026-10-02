//! The fakes for the connection and list queries: the current server, the server list, the
//! connection status, our client ID, the client and channel lists, and a client's channel.
//!
//! **Role:** `getCurrentServerConnectionHandlerID`, `getServerConnectionHandlerList`,
//! `getConnectionStatus`, `getClientID`, `getClientList`, `getChannelList`,
//! `getChannelClientList` and `getChannelOfClient`, answered from the fake state.
//!
//! **Position:** installed into the table by `fake_table`; the plugin calls them.
//!
//! **Signals and state:** each takes the fake-state lock once; called from any plugin thread.
//!
//! **Invariants:** lists are `malloc`ed, `0`-terminated and recorded in the ledger; an unknown
//! server answers `ERROR_not_connected`, an unknown client or channel its `invalid_id` error, as
//! TeamSpeak does; every call is logged as a [`HostQuery`].

use std::ffi::{c_int, c_uint};

use tfar_ts3_host::{ChannelId, ClientId, ServerConnectionHandlerId};

use super::fake_answers::{
    ERROR_CHANNEL_INVALID_ID, ERROR_CLIENT_INVALID_ID, ERROR_NOT_CONNECTED, hand_out_list,
    write_answer,
};
use crate::fake_state::host_session::with_fake_teamspeak;
use crate::{HostCall, HostQuery};

/// `getCurrentServerConnectionHandlerID`: the tab in front, `0` when none.
pub(crate) extern "C" fn get_current_server_connection_handler_id() -> u64 {
    with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::CurrentServerConnectionHandlerId));
        fake.current_server.0
    })
}

/// `getServerConnectionHandlerList`: every tab.
pub(crate) extern "C" fn get_server_connection_handler_list(result: *mut *mut u64) -> c_uint {
    with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::ServerConnectionHandlerList));
        let servers: Vec<u64> = fake.servers.keys().map(|server| server.0).collect();
        hand_out_list(
            &mut fake.ledger,
            "getServerConnectionHandlerList",
            &servers,
            result,
        )
    })
}

/// `getConnectionStatus`: the tab's status; an unknown tab is disconnected.
pub(crate) extern "C" fn get_connection_status(
    server_connection_handler_id: u64,
    result: *mut c_int,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    let status = with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::ConnectionStatus(server)));
        fake.server(server)
            .map_or(0, |fake_server| fake_server.connect_status.to_raw())
    });
    write_answer(result, status)
}

/// `getClientID`: our own client on the tab.
pub(crate) extern "C" fn get_client_id(
    server_connection_handler_id: u64,
    result: *mut u16,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    let own = with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::ClientId(server)));
        fake.server(server)
            .map(|fake_server| fake_server.own_client)
    });
    match own {
        Some(client) => write_answer(result, client.0),
        None => ERROR_NOT_CONNECTED,
    }
}

/// `getClientList`: every visible client on the tab.
pub(crate) extern "C" fn get_client_list(
    server_connection_handler_id: u64,
    result: *mut *mut u16,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::ClientList(server)));
        let Some(fake_server) = fake.server(server) else {
            return ERROR_NOT_CONNECTED;
        };
        let clients: Vec<u16> = fake_server.clients.keys().map(|client| client.0).collect();
        hand_out_list(&mut fake.ledger, "getClientList", &clients, result)
    })
}

/// `getChannelList`: every channel on the tab.
pub(crate) extern "C" fn get_channel_list(
    server_connection_handler_id: u64,
    result: *mut *mut u64,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::ChannelList(server)));
        let Some(fake_server) = fake.server(server) else {
            return ERROR_NOT_CONNECTED;
        };
        let channels: Vec<u64> = fake_server
            .channels
            .keys()
            .map(|channel| channel.0)
            .collect();
        hand_out_list(&mut fake.ledger, "getChannelList", &channels, result)
    })
}

/// `getChannelClientList`: the clients in one channel.
pub(crate) extern "C" fn get_channel_client_list(
    server_connection_handler_id: u64,
    channel_id: u64,
    result: *mut *mut u16,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    let channel = ChannelId(channel_id);
    with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::ChannelClientList(
            server, channel,
        )));
        let Some(fake_server) = fake.server(server) else {
            return ERROR_NOT_CONNECTED;
        };
        if !fake_server.channels.contains_key(&channel) {
            return ERROR_CHANNEL_INVALID_ID;
        }
        let clients: Vec<u16> = fake_server
            .clients
            .iter()
            .filter(|(_, client)| client.channel == channel)
            .map(|(client, _)| client.0)
            .collect();
        hand_out_list(&mut fake.ledger, "getChannelClientList", &clients, result)
    })
}

/// `getChannelOfClient`: the channel a client is in.
pub(crate) extern "C" fn get_channel_of_client(
    server_connection_handler_id: u64,
    client_id: u16,
    result: *mut u64,
) -> c_uint {
    let server = ServerConnectionHandlerId(server_connection_handler_id);
    let client = ClientId(client_id);
    let channel = with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::ChannelOfClient(server, client)));
        let fake_server = fake.server(server).ok_or(ERROR_NOT_CONNECTED)?;
        let fake_client = fake_server
            .clients
            .get(&client)
            .ok_or(ERROR_CLIENT_INVALID_ID)?;
        Ok(fake_client.channel)
    });
    match channel {
        Ok(channel) => write_answer(result, channel.0),
        Err(code) => code,
    }
}
