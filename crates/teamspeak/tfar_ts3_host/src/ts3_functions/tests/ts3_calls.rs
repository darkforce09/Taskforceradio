//! Unit tests for the [`Ts3`] methods against a fake TeamSpeak table: strings and lists are copied
//! and freed, error codes become `Ts3Error`s with TeamSpeak's text, arguments arrive as TeamSpeak
//! expects them.

use std::cell::RefCell;
use std::ffi::{CStr, c_char, c_int, c_uint};
use std::path::Path;

use ts3plugin_sys::public_definitions::LogLevel as SdkLogLevel;

use super::Ts3;
use crate::fake_ts3_table::{
    fake_error_text, fake_table, malloc_list, malloc_string, take_freed_pointers,
};
use crate::{
    ChannelId, ChannelProperty, ClientId, ClientProperty, LogLevel, PluginCommandTarget,
    ServerConnectionHandlerId, ServerProperty, Ts3Error, Ts3Vector,
};

thread_local! {
    /// What the fakes saw on this thread, one line per call.
    static CALLS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

fn record(call: String) {
    CALLS.with(|calls| calls.borrow_mut().push(call));
}

fn take_calls() -> Vec<String> {
    CALLS.with(|calls| std::mem::take(&mut *calls.borrow_mut()))
}

/// Reads a C string argument a fake receives.
fn text_argument(text: *const c_char) -> String {
    if text.is_null() {
        return "<null>".to_owned();
    }
    // SAFETY: the code under test passes NUL-terminated strings valid for the call.
    unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned()
}

/// Reads a 0-terminated `anyID` array argument a fake receives.
fn client_array_argument(mut cursor: *const u16) -> Vec<u16> {
    let mut ids = Vec::new();
    loop {
        // SAFETY: the code under test passes a 0-terminated array.
        let id = unsafe { cursor.read() };
        if id == 0 {
            return ids;
        }
        ids.push(id);
        // SAFETY: not yet at the terminator, so the next element is inside the array.
        cursor = unsafe { cursor.add(1) };
    }
}

// The addresses the fakes handed out, to check they are the ones freed.
thread_local! {
    static HANDED_OUT: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

extern "C" fn fake_client_nickname(
    server: u64,
    client: u16,
    flag: usize,
    result: *mut *mut c_char,
) -> c_uint {
    record(format!(
        "getClientVariableAsString {server} {client} {flag}"
    ));
    let text = malloc_string("Nkey \u{fc}");
    HANDED_OUT.with(|out| out.borrow_mut().push(text as usize));
    // SAFETY: the code under test passes a writable slot.
    unsafe { result.write(text) };
    0
}

extern "C" fn fake_failing_string(
    _server: u64,
    _client: u16,
    _flag: usize,
    _result: *mut *mut c_char,
) -> c_uint {
    0x0200
}

extern "C" fn fake_client_list(server: u64, result: *mut *mut u16) -> c_uint {
    record(format!("getClientList {server}"));
    let list = malloc_list(&[7_u16, 3, 12, 0]);
    HANDED_OUT.with(|out| out.borrow_mut().push(list as usize));
    // SAFETY: the code under test passes a writable slot.
    unsafe { result.write(list) };
    0
}

extern "C" fn fake_channel_list(_server: u64, result: *mut *mut u64) -> c_uint {
    let list = malloc_list(&[1_u64, 99, 0]);
    // SAFETY: the code under test passes a writable slot.
    unsafe { result.write(list) };
    0
}

extern "C" fn fake_null_channel_list(_server: u64, result: *mut *mut u64) -> c_uint {
    // SAFETY: the code under test passes a writable slot.
    unsafe { result.write(std::ptr::null_mut()) };
    0
}

extern "C" fn fake_handler_list(result: *mut *mut u64) -> c_uint {
    let list = malloc_list(&[4_u64, 0]);
    // SAFETY: the code under test passes a writable slot.
    unsafe { result.write(list) };
    0
}

extern "C" fn fake_connection_status(_server: u64, result: *mut c_int) -> c_uint {
    // SAFETY: the code under test passes a writable slot.
    unsafe { result.write(4) };
    0
}

extern "C" fn fake_mute(server: u64, clients: *const u16, return_code: *const c_char) -> c_uint {
    record(format!(
        "mute {server} {:?} {}",
        client_array_argument(clients),
        text_argument(return_code)
    ));
    0
}

extern "C" fn fake_unmute(server: u64, clients: *const u16, _return_code: *const c_char) -> c_uint {
    record(format!(
        "unmute {server} {:?}",
        client_array_argument(clients)
    ));
    0
}

extern "C" fn fake_send_plugin_command(
    server: u64,
    plugin_id: *const c_char,
    command: *const c_char,
    target_mode: c_int,
    targets: *const u16,
    _return_code: *const c_char,
) {
    let targets = if targets.is_null() {
        None
    } else {
        Some(client_array_argument(targets))
    };
    record(format!(
        "sendPluginCommand {server} {} {} {target_mode} {targets:?}",
        text_argument(plugin_id),
        text_argument(command)
    ));
}

extern "C" fn fake_flush_no_update(_server: u64, _return_code: *const c_char) -> c_uint {
    0x0003
}

extern "C" fn fake_set_self_string(server: u64, flag: usize, value: *const c_char) -> c_uint {
    record(format!(
        "setClientSelfVariableAsString {server} {flag} {}",
        text_argument(value)
    ));
    0
}

extern "C" fn fake_plugin_path(path: *mut c_char, max_length: usize, plugin_id: *const c_char) {
    record(format!(
        "getPluginPath {max_length} {}",
        text_argument(plugin_id)
    ));
    let text = b"/home/user/.ts3client/plugins\0";
    // SAFETY: the code under test passes a writable buffer of `max_length` bytes, far longer.
    unsafe { std::ptr::copy_nonoverlapping(text.as_ptr().cast::<c_char>(), path, text.len()) };
}

extern "C" fn fake_empty_config_path(_path: *mut c_char, _max_length: usize) {}

extern "C" fn fake_log_message(
    text: *const c_char,
    level: SdkLogLevel,
    channel: *const c_char,
    log_id: u64,
) -> c_uint {
    record(format!(
        "logMessage {} {level:?} {} {log_id}",
        text_argument(text),
        text_argument(channel)
    ));
    0
}

extern "C" fn fake_listener(
    _server: u64,
    position: *const ts3plugin_sys::public_definitions::Ts3Vector,
    forward: *const ts3plugin_sys::public_definitions::Ts3Vector,
    up: *const ts3plugin_sys::public_definitions::Ts3Vector,
) -> c_uint {
    // SAFETY: the code under test passes three readable vectors.
    let (position, forward, up) = unsafe { (&*position, &*forward, &*up) };
    record(format!(
        "listener {} {} {} {}",
        position.x, forward.y, up.z, position.z
    ));
    0
}

fn ts3_with(configure: impl FnOnce(&mut ts3plugin_sys::ts3functions::Ts3Functions)) -> Ts3 {
    let mut table = fake_table();
    configure(&mut table);
    let _ = take_calls();
    let _ = take_freed_pointers();
    Ts3::from_functions(table)
}

#[test]
fn a_teamspeak_string_is_copied_and_freed_exactly_once() {
    let ts3 = ts3_with(|table| table.get_client_variable_as_string = fake_client_nickname);
    let nickname = ts3
        .client_variable_as_string(
            ServerConnectionHandlerId(5),
            ClientId(9),
            ClientProperty::Nickname,
        )
        .unwrap();
    assert_eq!(nickname, "Nkey \u{fc}");
    assert_eq!(
        take_calls(),
        vec!["getClientVariableAsString 5 9 1".to_owned()]
    );
    let handed_out = HANDED_OUT.with(|out| out.borrow_mut().pop().unwrap());
    assert_eq!(take_freed_pointers(), vec![handed_out]);
}

#[test]
fn an_error_code_becomes_a_call_error_with_teamspeaks_text() {
    let ts3 = ts3_with(|table| table.get_client_variable_as_string = fake_failing_string);
    let error = ts3
        .client_variable_as_string(
            ServerConnectionHandlerId(1),
            ClientId(2),
            ClientProperty::MetaData,
        )
        .unwrap_err();
    assert_eq!(
        error,
        Ts3Error::Call {
            function: "getClientVariableAsString",
            code: 0x0200,
            message: fake_error_text(0x0200)
        }
    );
    // The error text TeamSpeak allocated is freed too.
    assert_eq!(take_freed_pointers().len(), 1);
}

#[test]
fn a_client_list_is_read_up_to_its_zero_terminator_and_freed() {
    let ts3 = ts3_with(|table| table.get_client_list = fake_client_list);
    let clients = ts3.client_list(ServerConnectionHandlerId(3)).unwrap();
    assert_eq!(clients, vec![ClientId(7), ClientId(3), ClientId(12)]);
    let handed_out = HANDED_OUT.with(|out| out.borrow_mut().pop().unwrap());
    assert_eq!(take_freed_pointers(), vec![handed_out]);
}

#[test]
fn channel_and_server_lists_become_their_newtypes() {
    let ts3 = ts3_with(|table| {
        table.get_channel_list = fake_channel_list;
        table.get_server_connection_handler_list = fake_handler_list;
        table.get_connection_status = fake_connection_status;
    });
    assert_eq!(
        ts3.channel_list(ServerConnectionHandlerId(1)).unwrap(),
        vec![ChannelId(1), ChannelId(99)]
    );
    assert_eq!(
        ts3.server_connection_handler_list().unwrap(),
        vec![ServerConnectionHandlerId(4)]
    );
    assert_eq!(
        ts3.connection_status(ServerConnectionHandlerId(4)).unwrap(),
        crate::ConnectStatus::ConnectionEstablished
    );
    assert_eq!(take_freed_pointers().len(), 2);
}

#[test]
fn a_null_list_is_empty_and_frees_nothing() {
    let ts3 = ts3_with(|table| table.get_channel_list = fake_null_channel_list);
    assert_eq!(
        ts3.channel_list(ServerConnectionHandlerId(1)).unwrap(),
        Vec::<ChannelId>::new()
    );
    assert_eq!(take_freed_pointers(), Vec::<usize>::new());
}

#[test]
fn mute_requests_pass_a_zero_terminated_array_and_no_return_code() {
    let ts3 = ts3_with(|table| {
        table.request_mute_clients = fake_mute;
        table.requset_unmute_clients = fake_unmute;
    });
    ts3.request_mute_clients(ServerConnectionHandlerId(2), &[ClientId(4), ClientId(5)])
        .unwrap();
    ts3.request_unmute_clients(ServerConnectionHandlerId(2), &[ClientId(6)])
        .unwrap();
    assert_eq!(
        take_calls(),
        vec!["mute 2 [4, 5] <null>".to_owned(), "unmute 2 [6]".to_owned()]
    );
}

#[test]
fn an_empty_mute_list_never_reaches_teamspeak() {
    // The mute slots keep the aborting stub: a call would end the test run.
    let ts3 = ts3_with(|_| {});
    ts3.request_mute_clients(ServerConnectionHandlerId(2), &[])
        .unwrap();
    ts3.request_unmute_clients(ServerConnectionHandlerId(2), &[])
        .unwrap();
}

#[test]
fn a_plugin_command_needs_the_registered_plugin_id() {
    let ts3 = ts3_with(|table| table.send_plugin_command = fake_send_plugin_command);
    let error = ts3
        .send_plugin_command(
            ServerConnectionHandlerId(1),
            "VOLUME",
            PluginCommandTarget::CurrentChannel,
        )
        .unwrap_err();
    assert_eq!(
        error,
        Ts3Error::PluginIdNotRegistered {
            function: "sendPluginCommand"
        }
    );
    assert_eq!(take_calls(), Vec::<String>::new());
}

#[test]
fn a_plugin_command_carries_the_id_the_mode_and_the_target_clients() {
    let ts3 = ts3_with(|table| table.send_plugin_command = fake_send_plugin_command);
    ts3.register_plugin_id(c"tfar_plugin_1_0");
    ts3.send_plugin_command(
        ServerConnectionHandlerId(1),
        "TANGENT\tPRESSED",
        PluginCommandTarget::CurrentChannel,
    )
    .unwrap();
    ts3.send_plugin_command(
        ServerConnectionHandlerId(1),
        "REQVOL\t7",
        PluginCommandTarget::Clients(&[ClientId(7), ClientId(8)]),
    )
    .unwrap();
    assert_eq!(
        take_calls(),
        vec![
            "sendPluginCommand 1 tfar_plugin_1_0 TANGENT\tPRESSED 0 None".to_owned(),
            "sendPluginCommand 1 tfar_plugin_1_0 REQVOL\t7 2 Some([7, 8])".to_owned(),
        ]
    );
}

#[test]
fn text_with_a_nul_byte_is_refused_before_teamspeak_sees_it() {
    let ts3 = ts3_with(|table| table.set_client_self_variable_as_string = fake_set_self_string);
    let error = ts3
        .set_client_self_variable_as_string(
            ServerConnectionHandlerId(1),
            ClientProperty::MetaData,
            "a\0b",
        )
        .unwrap_err();
    assert_eq!(
        error,
        Ts3Error::InteriorNul {
            function: "setClientSelfVariableAsString",
            position: 1
        }
    );
    assert_eq!(take_calls(), Vec::<String>::new());
    ts3.set_client_self_variable_as_string(
        ServerConnectionHandlerId(1),
        ClientProperty::MetaData,
        "<TFAR>",
    )
    .unwrap();
    assert_eq!(
        take_calls(),
        vec!["setClientSelfVariableAsString 1 15 <TFAR>".to_owned()]
    );
}

#[test]
fn ok_no_update_counts_as_success() {
    let ts3 = ts3_with(|table| table.flush_client_self_updates = fake_flush_no_update);
    assert_eq!(
        ts3.flush_client_self_updates(ServerConnectionHandlerId(1)),
        Ok(())
    );
}

#[test]
fn the_plugin_path_uses_the_three_argument_form_with_the_plugin_id() {
    let ts3 = ts3_with(|table| table.get_plugin_path = fake_plugin_path);
    assert_eq!(
        ts3.plugin_path(),
        Err(Ts3Error::PluginIdNotRegistered {
            function: "getPluginPath"
        })
    );
    ts3.register_plugin_id(c"tfar_id");
    assert_eq!(
        ts3.plugin_path().unwrap(),
        Path::new("/home/user/.ts3client/plugins")
    );
    assert_eq!(take_calls(), vec!["getPluginPath 4096 tfar_id".to_owned()]);
}

#[test]
fn an_empty_config_path_is_an_error() {
    let ts3 = ts3_with(|table| table.get_config_path = fake_empty_config_path);
    assert_eq!(
        ts3.config_path(),
        Err(Ts3Error::EmptyPath {
            function: "getConfigPath"
        })
    );
}

#[test]
fn a_log_message_passes_level_channel_and_server() {
    let ts3 = ts3_with(|table| table.log_message = fake_log_message);
    ts3.log_message(
        "hello",
        LogLevel::Warning,
        "task_force_radio",
        ServerConnectionHandlerId(141),
    )
    .unwrap();
    assert_eq!(
        take_calls(),
        vec!["logMessage hello Warning task_force_radio 141".to_owned()]
    );
}

#[test]
fn listener_vectors_arrive_in_their_slots() {
    let ts3 = ts3_with(|table| table.systemset3d_listener_attributes = fake_listener);
    let position = Ts3Vector {
        x: 1.0,
        y: 0.0,
        z: 3.0,
    };
    let forward = Ts3Vector {
        x: 0.0,
        y: 2.0,
        z: 0.0,
    };
    let up = Ts3Vector {
        x: 0.0,
        y: 0.0,
        z: 5.0,
    };
    ts3.system_set_3d_listener_attributes(ServerConnectionHandlerId(1), position, forward, up)
        .unwrap();
    assert_eq!(take_calls(), vec!["listener 1 2 5 3".to_owned()]);
}

#[test]
fn property_flags_reach_teamspeak_unchanged() {
    // Values from `public_definitions.h`; `sdk_layout` checks every flag against the header.
    assert_eq!(ClientProperty::FlagTalking.flag(), 4);
    assert_eq!(ChannelProperty::Name.flag(), 0);
    assert_eq!(ServerProperty::Name.flag(), 1);
}

extern "C" fn fake_listener_null_aware(
    _server: u64,
    position: *const ts3plugin_sys::public_definitions::Ts3Vector,
    forward: *const ts3plugin_sys::public_definitions::Ts3Vector,
    up: *const ts3plugin_sys::public_definitions::Ts3Vector,
) -> c_uint {
    let describe = |vector: *const ts3plugin_sys::public_definitions::Ts3Vector| {
        if vector.is_null() {
            "null".to_owned()
        } else {
            // SAFETY: a non-null vector argument points at a readable vector for the call.
            let vector = unsafe { &*vector };
            format!("{} {} {}", vector.x, vector.y, vector.z)
        }
    };
    record(format!(
        "listener [{}] [{}] [{}]",
        describe(position),
        describe(forward),
        describe(up)
    ));
    0
}

#[test]
fn absent_listener_orientation_vectors_reach_teamspeak_as_null() {
    let ts3 = ts3_with(|table| table.systemset3d_listener_attributes = fake_listener_null_aware);
    let position = Ts3Vector {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    ts3.system_set_3d_listener_attributes(ServerConnectionHandlerId(1), position, None, None)
        .unwrap();
    assert_eq!(
        take_calls(),
        vec!["listener [1 2 3] [null] [null]".to_owned()]
    );
}

#[test]
fn listener_vectors_may_be_given_bare_or_as_options_and_mixed_with_null() {
    let ts3 = ts3_with(|table| table.systemset3d_listener_attributes = fake_listener_null_aware);
    let origin = Ts3Vector::default();
    let forward = Ts3Vector {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    ts3.system_set_3d_listener_attributes(ServerConnectionHandlerId(1), origin, forward, None)
        .unwrap();
    ts3.system_set_3d_listener_attributes(
        ServerConnectionHandlerId(1),
        origin,
        None,
        Some(forward),
    )
    .unwrap();
    assert_eq!(
        take_calls(),
        vec![
            "listener [0 0 0] [0 1 0] [null]".to_owned(),
            "listener [0 0 0] [null] [0 1 0]".to_owned(),
        ]
    );
}
