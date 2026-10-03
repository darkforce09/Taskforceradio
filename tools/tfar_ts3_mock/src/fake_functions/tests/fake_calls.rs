//! Unit tests of the typed fakes, called through the table's function pointers as a plugin
//! calls them: answers, string and list ownership, the call log and the leak report.

use std::collections::BTreeMap;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::path::PathBuf;

use tfar_ts3_host::{ChannelId, ClientId, ConnectStatus, LogLevel, ServerConnectionHandlerId};
use ts3plugin_sys::public_definitions as sdk;

use super::*;
use crate::fake_functions::fake_answers::{
    ERROR_CLIENT_INVALID_ID, ERROR_NOT_CONNECTED, ERROR_OK, ERROR_PARAMETER_INVALID,
};
use crate::fake_state::host_session::HostSession;
use crate::{FakeChannel, FakeClient, FakeServer, HostQuery, SentCommandTarget};

const SERVER: ServerConnectionHandlerId = ServerConnectionHandlerId(1);
const OWN: ClientId = ClientId(1);
const OTHER: ClientId = ClientId(2);
const LOBBY: ChannelId = ChannelId(1);
const BRIEFING: ChannelId = ChannelId(2);

/// A session with server 1: us (client 1) and "Other" (client 2) in the lobby, an empty
/// briefing channel, and the plugin and config folders `/plugins` and `/config`.
fn session_with_one_server() -> HostSession {
    let session = HostSession::begin(
        Some(PathBuf::from("/plugins")),
        Some(PathBuf::from("/config")),
    )
    .unwrap();
    with_fake_teamspeak(|fake| {
        let mut other = FakeClient::new("Other", "other-uid=", LOBBY);
        other.metadata = "<TFAR>meta</TFAR>".to_owned();
        other.is_talking = true;
        fake.servers.insert(
            SERVER,
            FakeServer {
                name: "Test Server".to_owned(),
                unique_identifier: "server-uid=".to_owned(),
                connect_status: ConnectStatus::ConnectionEstablished,
                own_client: OWN,
                channels: BTreeMap::from([
                    (
                        LOBBY,
                        FakeChannel {
                            name: "Lobby".to_owned(),
                            parent: ChannelId(0),
                        },
                    ),
                    (
                        BRIEFING,
                        FakeChannel {
                            name: "Briefing".to_owned(),
                            parent: ChannelId(0),
                        },
                    ),
                ]),
                clients: BTreeMap::from([
                    (OWN, FakeClient::new("Me", "me-uid=", LOBBY)),
                    (OTHER, other),
                ]),
            },
        );
        fake.current_server = SERVER;
    });
    session
}

/// Copies the C string at `text` the plugin received, without freeing it.
fn read_text(text: *const c_char) -> String {
    assert!(!text.is_null());
    // SAFETY: the fakes hand out NUL-terminated strings that stay valid until freed.
    unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned()
}

/// Copies the zero-terminated list at `list` the plugin received, without freeing it.
fn read_list<Item: Copy + Default + PartialEq>(list: *const Item) -> Vec<Item> {
    assert!(!list.is_null());
    let mut items = Vec::new();
    for index in 0.. {
        // SAFETY: the fakes end every list in its `0` terminator, where the loop stops.
        let item = unsafe { list.add(index).read() };
        if item == Item::default() {
            break;
        }
        items.push(item);
    }
    items
}

fn calls() -> Vec<HostCall> {
    with_fake_teamspeak(|fake| fake.calls().to_vec())
}

fn report() -> crate::LeakReport {
    with_fake_teamspeak(|fake| fake.ledger.report())
}

#[test]
fn a_string_answer_is_malloced_recorded_and_freed_by_free_memory() {
    let _session = session_with_one_server();
    let table = fake_function_table();
    let mut nickname: *mut c_char = std::ptr::null_mut();
    let flag = sdk::ClientProperties::Nickname as usize;
    let answer = (table.get_client_variable_as_string)(SERVER.0, OTHER.0, flag, &raw mut nickname);
    assert_eq!(answer, ERROR_OK);
    assert_eq!(read_text(nickname), "Other");
    let outstanding = report().unfreed_host_allocations;
    assert_eq!(outstanding.len(), 1);
    assert_eq!(outstanding[0].function, "getClientVariableAsString");
    assert_eq!(outstanding[0].address, nickname as usize);

    assert_eq!((table.free_memory)(nickname.cast::<c_void>()), ERROR_OK);
    assert!(report().is_clean());
    assert_eq!(
        calls(),
        vec![HostCall::Query(HostQuery::ClientVariableAsString(
            SERVER, OTHER, flag
        ))]
    );
}

#[test]
fn an_answer_the_plugin_never_frees_is_a_leak() {
    let _session = session_with_one_server();
    let table = fake_function_table();
    let mut text: *mut c_char = std::ptr::null_mut();
    assert_eq!((table.get_error_message)(0x0702, &raw mut text), ERROR_OK);
    assert_eq!(read_text(text), "mock TeamSpeak error 0x0702");
    let leaks = report();
    assert!(!leaks.is_clean());
    assert_eq!(
        leaks.unfreed_host_allocations[0].function,
        "getErrorMessage"
    );
}

#[test]
fn freeing_an_unknown_or_freed_pointer_is_recorded_and_not_freed() {
    let _session = session_with_one_server();
    let table = fake_function_table();
    let mut not_from_teamspeak = 5_u64;
    let foreign = (&raw mut not_from_teamspeak).cast::<c_void>();
    (table.free_memory)(foreign);
    assert_eq!(not_from_teamspeak, 5);

    let mut list: *mut u64 = std::ptr::null_mut();
    assert_eq!(
        (table.get_server_connection_handler_list)(&raw mut list),
        ERROR_OK
    );
    (table.free_memory)(list.cast::<c_void>());
    (table.free_memory)(list.cast::<c_void>());
    (table.free_memory)(std::ptr::null_mut());

    let leaks = report();
    assert_eq!(leaks.unfreed_host_allocations, Vec::new());
    assert_eq!(leaks.unknown_frees, vec![foreign as usize, list as usize]);
    assert!(!leaks.is_clean());
}

#[test]
fn lists_are_zero_terminated_and_follow_the_fake_state() {
    let _session = session_with_one_server();
    let table = fake_function_table();
    let mut clients: *mut u16 = std::ptr::null_mut();
    assert_eq!(
        (table.get_client_list)(SERVER.0, &raw mut clients),
        ERROR_OK
    );
    assert_eq!(read_list(clients), vec![1, 2]);
    let mut channels: *mut u64 = std::ptr::null_mut();
    assert_eq!(
        (table.get_channel_list)(SERVER.0, &raw mut channels),
        ERROR_OK
    );
    assert_eq!(read_list(channels), vec![1, 2]);
    let mut in_briefing: *mut u16 = std::ptr::null_mut();
    let answer = (table.get_channel_client_list)(SERVER.0, BRIEFING.0, &raw mut in_briefing);
    assert_eq!(answer, ERROR_OK);
    assert_eq!(read_list(in_briefing), Vec::<u16>::new());
    let mut servers: *mut u64 = std::ptr::null_mut();
    assert_eq!(
        (table.get_server_connection_handler_list)(&raw mut servers),
        ERROR_OK
    );
    assert_eq!(read_list(servers), vec![1]);
    for block in [
        clients.cast::<c_void>(),
        channels.cast(),
        in_briefing.cast(),
        servers.cast(),
    ] {
        (table.free_memory)(block);
    }
    assert!(report().is_clean());
}

#[test]
fn queries_answer_from_the_fake_state() {
    let _session = session_with_one_server();
    let table = fake_function_table();
    assert_eq!((table.get_current_server_connection_handler_id)(), SERVER.0);
    let mut own = 0_u16;
    assert_eq!((table.get_client_id)(SERVER.0, &raw mut own), ERROR_OK);
    assert_eq!(own, OWN.0);
    let mut status: c_int = -1;
    assert_eq!(
        (table.get_connection_status)(SERVER.0, &raw mut status),
        ERROR_OK
    );
    assert_eq!(status, ConnectStatus::ConnectionEstablished.to_raw());
    let mut channel = 0_u64;
    assert_eq!(
        (table.get_channel_of_client)(SERVER.0, OTHER.0, &raw mut channel),
        ERROR_OK
    );
    assert_eq!(channel, LOBBY.0);
    let mut talking: c_int = 0;
    let flag = sdk::ClientProperties::FlagTalking as usize;
    assert_eq!(
        (table.get_client_variable_as_int)(SERVER.0, OTHER.0, flag, &raw mut talking),
        ERROR_OK
    );
    assert_eq!(talking, 1);
    let mut name: *mut c_char = std::ptr::null_mut();
    let channel_name = sdk::ChannelProperties::Name as usize;
    assert_eq!(
        (table.get_channel_variable_as_string)(SERVER.0, BRIEFING.0, channel_name, &raw mut name),
        ERROR_OK
    );
    assert_eq!(read_text(name), "Briefing");
    (table.free_memory)(name.cast());
    let mut server_name: *mut c_char = std::ptr::null_mut();
    let flag = sdk::VirtualServerProperties::Name as usize;
    assert_eq!(
        (table.get_server_variable_as_string)(SERVER.0, flag, &raw mut server_name),
        ERROR_OK
    );
    assert_eq!(read_text(server_name), "Test Server");
    (table.free_memory)(server_name.cast());
    let mut metadata: *mut c_char = std::ptr::null_mut();
    let flag = sdk::ClientProperties::MetaData as usize;
    assert_eq!(
        (table.get_client_variable_as_string)(SERVER.0, OTHER.0, flag, &raw mut metadata),
        ERROR_OK
    );
    assert_eq!(read_text(metadata), "<TFAR>meta</TFAR>");
    (table.free_memory)(metadata.cast());
    assert!(report().is_clean());
}

#[test]
fn unknown_servers_clients_and_flags_answer_teamspeak_errors() {
    let _session = session_with_one_server();
    let table = fake_function_table();
    let mut own = 0_u16;
    assert_eq!((table.get_client_id)(9, &raw mut own), ERROR_NOT_CONNECTED);
    let mut channel = 0_u64;
    assert_eq!(
        (table.get_channel_of_client)(SERVER.0, 77, &raw mut channel),
        ERROR_CLIENT_INVALID_ID
    );
    let mut text: *mut c_char = std::ptr::null_mut();
    let unmodelled = sdk::ClientProperties::Platform as usize;
    assert_eq!(
        (table.get_client_variable_as_string)(SERVER.0, OTHER.0, unmodelled, &raw mut text),
        ERROR_PARAMETER_INVALID
    );
    assert!(text.is_null());
    let flag = sdk::ClientProperties::Nickname as usize;
    assert_eq!(
        (table.get_client_variable_as_string)(SERVER.0, OTHER.0, flag, std::ptr::null_mut()),
        ERROR_PARAMETER_INVALID
    );
    assert!(report().is_clean());
}

#[test]
fn paths_end_in_a_slash_and_are_truncated_to_the_buffer() {
    let _session = session_with_one_server();
    let table = fake_function_table();
    let mut buffer = [0x55_u8; 64];
    let plugin_id = c"mock_plugin_id";
    (table.get_plugin_path)(buffer.as_mut_ptr().cast(), buffer.len(), plugin_id.as_ptr());
    assert_eq!(read_text(buffer.as_ptr().cast()), "/plugins/");
    (table.get_config_path)(buffer.as_mut_ptr().cast(), buffer.len());
    assert_eq!(read_text(buffer.as_ptr().cast()), "/config/");
    let mut small = [0x55_u8; 4];
    (table.get_config_path)(small.as_mut_ptr().cast(), small.len());
    assert_eq!(small, *b"/co\0");
    assert_eq!(
        calls(),
        vec![
            HostCall::Query(HostQuery::PluginPath),
            HostCall::Query(HostQuery::ConfigPath),
            HostCall::Query(HostQuery::ConfigPath),
        ]
    );
}

#[test]
fn the_plugin_path_is_empty_for_an_id_that_is_not_the_registered_one() {
    let _session = session_with_one_server();
    let table = fake_function_table();
    let mut buffer = [0x55_u8; 64];
    let stale_id = c"stale_plugin_id";
    (table.get_plugin_path)(buffer.as_mut_ptr().cast(), buffer.len(), stale_id.as_ptr());
    assert_eq!(read_text(buffer.as_ptr().cast()), "");
    with_fake_teamspeak(|fake| "stale_plugin_id".clone_into(&mut fake.registered_plugin_id));
    (table.get_plugin_path)(buffer.as_mut_ptr().cast(), buffer.len(), stale_id.as_ptr());
    assert_eq!(read_text(buffer.as_ptr().cast()), "/plugins/");
    assert_eq!(
        calls(),
        vec![
            HostCall::Query(HostQuery::PluginPath),
            HostCall::Query(HostQuery::PluginPath),
        ]
    );
}

#[test]
fn requests_are_logged_with_their_arguments_and_mutes_change_the_state() {
    let _session = session_with_one_server();
    let table = fake_function_table();
    let targets = [OTHER.0, 0];
    let plugin_id = c"mock_plugin_id";
    let command = c"TANGENT\tPRESSED";
    (table.send_plugin_command)(
        SERVER.0,
        plugin_id.as_ptr(),
        command.as_ptr(),
        sdk::PluginTargetMode::Client as c_int,
        targets.as_ptr(),
        std::ptr::null(),
    );
    (table.send_plugin_command)(
        SERVER.0,
        plugin_id.as_ptr(),
        command.as_ptr(),
        sdk::PluginTargetMode::Server as c_int,
        std::ptr::null(),
        std::ptr::null(),
    );
    assert_eq!(
        (table.request_mute_clients)(SERVER.0, targets.as_ptr(), std::ptr::null()),
        0
    );
    let is_muted = || with_fake_teamspeak(|fake| fake.client(SERVER, OTHER).unwrap().is_muted);
    assert!(is_muted());
    assert_eq!(
        (table.requset_unmute_clients)(SERVER.0, targets.as_ptr(), std::ptr::null()),
        0
    );
    assert!(!is_muted());
    let password = c"secret";
    let answer = (table.request_client_move)(
        SERVER.0,
        OWN.0,
        BRIEFING.0,
        password.as_ptr(),
        std::ptr::null(),
    );
    assert_eq!(answer, ERROR_OK);
    let wave = c"/plugins/radio-sounds/on.wav";
    assert_eq!((table.play_wave_file)(SERVER.0, wave.as_ptr()), ERROR_OK);
    assert_eq!(
        calls(),
        vec![
            HostCall::SendPluginCommand {
                server: SERVER,
                plugin_id: "mock_plugin_id".to_owned(),
                command: "TANGENT\tPRESSED".to_owned(),
                target: SentCommandTarget::Clients(vec![OTHER]),
            },
            HostCall::SendPluginCommand {
                server: SERVER,
                plugin_id: "mock_plugin_id".to_owned(),
                command: "TANGENT\tPRESSED".to_owned(),
                target: SentCommandTarget::Server,
            },
            HostCall::RequestMuteClients {
                server: SERVER,
                clients: vec![OTHER]
            },
            HostCall::RequestUnmuteClients {
                server: SERVER,
                clients: vec![OTHER]
            },
            HostCall::RequestClientMove {
                server: SERVER,
                client: OWN,
                channel: BRIEFING,
                password: "secret".to_owned(),
            },
            HostCall::PlayWaveFile {
                server: SERVER,
                path: PathBuf::from("/plugins/radio-sounds/on.wav"),
            },
        ]
    );
    with_fake_teamspeak(|fake| {
        assert_eq!(
            fake.client(SERVER, OWN).unwrap().channel,
            LOBBY,
            "a move request moves nobody"
        );
    });
}

#[test]
fn own_variables_are_set_read_and_flushed() {
    let _session = session_with_one_server();
    let table = fake_function_table();
    let metadata = CString::new("<TFAR>on</TFAR>").unwrap();
    let flag = sdk::ClientProperties::MetaData as usize;
    assert_eq!(
        (table.set_client_self_variable_as_string)(SERVER.0, flag, metadata.as_ptr()),
        ERROR_OK
    );
    let deactivated = sdk::ClientProperties::InputDeactivated as usize;
    assert_eq!(
        (table.set_client_self_variable_as_int)(SERVER.0, deactivated, 1),
        ERROR_OK
    );
    assert_eq!(
        (table.flush_client_self_updates)(SERVER.0, std::ptr::null()),
        ERROR_OK
    );
    let mut value: c_int = 0;
    assert_eq!(
        (table.get_client_self_variable_as_int)(SERVER.0, deactivated, &raw mut value),
        ERROR_OK
    );
    assert_eq!(value, 1);
    with_fake_teamspeak(|fake| {
        assert_eq!(
            fake.client(SERVER, OWN).unwrap().metadata,
            "<TFAR>on</TFAR>"
        );
    });
    let unwritable = sdk::ClientProperties::Platform as usize;
    assert_eq!(
        (table.set_client_self_variable_as_int)(SERVER.0, unwritable, 1),
        ERROR_PARAMETER_INVALID
    );
}

#[test]
fn pre_processor_values_round_trip_and_vad_starts_off() {
    let _session = session_with_one_server();
    let table = fake_function_table();
    let read = |ident: &CStr| {
        let mut value: *mut c_char = std::ptr::null_mut();
        let answer =
            (table.get_pre_processor_config_value)(SERVER.0, ident.as_ptr(), &raw mut value);
        if answer != ERROR_OK {
            return Err(answer);
        }
        let text = read_text(value);
        (table.free_memory)(value.cast());
        Ok(text)
    };
    assert_eq!(read(c"vad"), Ok("false".to_owned()));
    assert_eq!(
        (table.set_pre_processor_config_value)(SERVER.0, c"vad".as_ptr(), c"true".as_ptr()),
        0
    );
    assert_eq!(read(c"vad"), Ok("true".to_owned()));
    assert_eq!(read(c"unknown_setting"), Err(ERROR_PARAMETER_INVALID));
    assert!(report().is_clean());
}

#[test]
fn log_lines_chat_lines_and_3d_calls_are_logged() {
    let _session = session_with_one_server();
    let table = fake_function_table();
    let answer = (table.log_message)(
        c"radio on".as_ptr(),
        sdk::LogLevel::Warning,
        c"task_force_radio".as_ptr(),
        1,
    );
    assert_eq!(answer, ERROR_OK);
    (table.print_message_to_current_tab)(c"[b]TFAR[/b]".as_ptr());
    assert_eq!((table.systemset3d_settings)(SERVER.0, 1.0, 0.5), ERROR_OK);
    let position = sdk::Ts3Vector {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    assert_eq!(
        (table.channelset3d_attributes)(SERVER.0, OTHER.0, &raw const position),
        ERROR_OK
    );
    let answer = (table.systemset3d_listener_attributes)(
        SERVER.0,
        &raw const position,
        std::ptr::null(),
        std::ptr::null(),
    );
    assert_eq!(answer, ERROR_OK);
    let vector = tfar_ts3_host::Ts3Vector {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    assert_eq!(
        calls(),
        vec![
            HostCall::LogMessage {
                text: "radio on".to_owned(),
                level: LogLevel::Warning,
                channel: "task_force_radio".to_owned(),
                server: SERVER,
            },
            HostCall::PrintMessageToCurrentTab {
                text: "[b]TFAR[/b]".to_owned()
            },
            HostCall::Systemset3DSettings {
                server: SERVER,
                distance_factor: 1.0,
                rolloff_scale: 0.5,
            },
            HostCall::Channelset3DAttributes {
                server: SERVER,
                client: OTHER,
                position: Some(vector),
            },
            HostCall::Systemset3DListenerAttributes {
                server: SERVER,
                position: Some(vector),
                forward: None,
                up: None,
            },
        ]
    );
}

#[test]
fn a_new_session_starts_from_fresh_state_and_frees_what_was_left() {
    {
        let _session = session_with_one_server();
        let table = fake_function_table();
        let mut text: *mut c_char = std::ptr::null_mut();
        (table.get_error_message)(1, &raw mut text);
        assert!(!report().is_clean());
    }
    let _session = HostSession::begin(None, None).unwrap();
    with_fake_teamspeak(|fake| {
        assert!(fake.servers.is_empty());
        assert_eq!(fake.calls(), &[]);
        assert!(fake.plugin_path.ends_with("plugins") && fake.plugin_path.is_dir());
        assert!(fake.config_path.ends_with("config") && fake.config_path.is_dir());
        assert_eq!(
            fake.preprocessor_values.get("vad").map(String::as_str),
            Some("false")
        );
    });
    assert!(report().is_clean());
}

#[test]
fn listener_attributes_accept_a_null_for_any_vector_and_log_it_as_none() {
    let _session = session_with_one_server();
    let table = fake_function_table();
    let forward = sdk::Ts3Vector {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    let up = sdk::Ts3Vector {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    let answers = [
        (table.systemset3d_listener_attributes)(
            SERVER.0,
            std::ptr::null(),
            &raw const forward,
            &raw const up,
        ),
        (table.systemset3d_listener_attributes)(
            SERVER.0,
            &raw const up,
            std::ptr::null(),
            &raw const up,
        ),
        (table.systemset3d_listener_attributes)(
            SERVER.0,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        ),
    ];
    assert_eq!(answers, [ERROR_OK; 3]);
    let typed = |vector: &sdk::Ts3Vector| tfar_ts3_host::Ts3Vector {
        x: vector.x,
        y: vector.y,
        z: vector.z,
    };
    let listener = |position, forward, up| HostCall::Systemset3DListenerAttributes {
        server: SERVER,
        position,
        forward,
        up,
    };
    assert_eq!(
        calls(),
        vec![
            listener(None, Some(typed(&forward)), Some(typed(&up))),
            listener(Some(typed(&up)), None, Some(typed(&up))),
            listener(None, None, None),
        ]
    );
    assert!(report().is_clean());
}
