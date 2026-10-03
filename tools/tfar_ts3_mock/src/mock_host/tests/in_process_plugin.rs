//! An in-process stand-in for a plugin: `extern "C"` exports defined in the test binary, handed
//! to [`MockHost`] as a [`PluginLibrary`] over the test executable itself.
//!
//! The exports record what TeamSpeak tells them and call back into the fake table the way TFAR
//! does (paths, log lines, client queries with `freeMemory`), so the host's start order, drivers
//! and memory rules are tested without a separately built plugin.

use std::ffi::{CStr, c_char, c_int, c_uint, c_void};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

use parking_lot::{Mutex, MutexGuard};
use ts3plugin_sys::public_definitions as sdk;
use ts3plugin_sys::ts3functions::Ts3Functions;

use crate::{HostOptions, MockHost, PluginExports, PluginLibrary, Result};

/// What the plugin was told, in order.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PluginEvent {
    RegisteredPluginId(String),
    Init {
        config_path: String,
    },
    Shutdown,
    CurrentServerChanged(u64),
    ConnectStatus {
        server: u64,
        status: c_int,
        own_client: Option<u16>,
    },
    NewChannel {
        server: u64,
        channel: u64,
        parent: u64,
    },
    ClientMove {
        server: u64,
        client: u16,
        old_channel: u64,
        new_channel: u64,
        visibility: c_int,
        message: String,
        nickname_seen: Option<String>,
    },
    UpdateClient {
        server: u64,
        client: u16,
        invoker: u16,
        invoker_name: String,
    },
    PluginCommand {
        server: u64,
        plugin_name: String,
        command: String,
        invoker: u16,
        invoker_name: String,
        invoker_unique_identifier: String,
    },
    TalkStatus {
        server: u64,
        status: c_int,
        is_received_whisper: c_int,
        client: u16,
        flag_talking_seen: Option<c_int>,
    },
}

/// The table the mock handed over.
static TABLE: Mutex<Option<Ts3Functions>> = Mutex::new(None);
/// The events seen.
static EVENTS: Mutex<Vec<PluginEvent>> = Mutex::new(Vec::new());
/// What `init` answers.
static INIT_ANSWER: AtomicI32 = AtomicI32::new(0);
/// How many `infoData` blocks came back through `freeMemory`.
static FREED_INFO_BLOCKS: AtomicUsize = AtomicUsize::new(0);
/// Serialises the tests that use this plugin's statics.
static IN_PROCESS_PLUGIN_USERS: Mutex<()> = Mutex::new(());

fn record(event: PluginEvent) {
    EVENTS.lock().push(event);
}

fn text(pointer: *const c_char) -> String {
    if pointer.is_null() {
        return String::new();
    }
    // SAFETY: the mock passes NUL-terminated strings valid for the call.
    unsafe { CStr::from_ptr(pointer) }
        .to_string_lossy()
        .into_owned()
}

/// A copy of one table slot, taken without holding the lock across the call.
fn slot<Function: Copy>(pick: impl FnOnce(&Ts3Functions) -> Function) -> Function {
    let table = TABLE.lock();
    pick(table.as_ref().unwrap())
}

extern "C" fn name() -> *const c_char {
    c"In-process Test Plugin".as_ptr()
}
extern "C" fn version() -> *const c_char {
    c"1.2.3".as_ptr()
}
extern "C" fn api_version() -> c_int {
    26
}
pub(crate) extern "C" fn old_api_version() -> c_int {
    25
}
extern "C" fn author() -> *const c_char {
    c"Test Author".as_ptr()
}
extern "C" fn description() -> *const c_char {
    c"Records what the mock does".as_ptr()
}
extern "C" fn set_function_pointers(functions: Ts3Functions) {
    *TABLE.lock() = Some(functions);
}
extern "C" fn init() -> c_int {
    let get_config_path = slot(|table| table.get_config_path);
    let mut buffer = [0_u8; 256];
    get_config_path(buffer.as_mut_ptr().cast(), buffer.len());
    let log_message = slot(|table| table.log_message);
    log_message(
        c"init".as_ptr(),
        sdk::LogLevel::Info,
        c"test_plugin".as_ptr(),
        0,
    );
    record(PluginEvent::Init {
        config_path: text(buffer.as_ptr().cast()),
    });
    INIT_ANSWER.load(Ordering::SeqCst)
}
extern "C" fn shutdown() {
    record(PluginEvent::Shutdown);
}
extern "C" fn register_plugin_id(id: *const c_char) {
    record(PluginEvent::RegisteredPluginId(text(id)));
}
extern "C" fn request_autoload() -> c_int {
    1
}
extern "C" fn command_keyword() -> *const c_char {
    c"test".as_ptr()
}
extern "C" fn process_command(_server: u64, command: *const c_char) -> c_int {
    if text(command) == "ping" {
        let print = slot(|table| table.print_message_to_current_tab);
        print(c"pong".as_ptr());
        0
    } else {
        1
    }
}
extern "C" fn current_server_connection_changed(server: u64) {
    record(PluginEvent::CurrentServerChanged(server));
}
extern "C" fn info_data(_server: u64, id: u64, item_type: c_int, data: *mut *mut c_char) {
    let answer = format!("info {item_type} {id}\0");
    // SAFETY: `malloc` has no precondition; the block is checked and filled below.
    let block = unsafe { libc::malloc(answer.len()) }.cast::<c_char>();
    assert!(!block.is_null());
    // SAFETY: `block` holds `answer.len()` bytes; `data` is the writable slot the mock passed.
    unsafe {
        std::ptr::copy_nonoverlapping(answer.as_ptr().cast::<c_char>(), block, answer.len());
        data.write(block);
    }
}
extern "C" fn free_memory(block: *mut c_void) {
    FREED_INFO_BLOCKS.fetch_add(1, Ordering::SeqCst);
    // SAFETY: the mock hands back only the `malloc`ed `infoData` blocks, once each.
    unsafe { libc::free(block) };
}
extern "C" fn on_connect_status_change(server: u64, status: c_int, _error: c_uint) {
    let mut own_client = None;
    if status == sdk::ConnectStatus::ConnectionEstablished as c_int {
        let get_client_id = slot(|table| table.get_client_id);
        let mut client = 0_u16;
        if get_client_id(server, &raw mut client) == 0 {
            own_client = Some(client);
        }
    }
    record(PluginEvent::ConnectStatus {
        server,
        status,
        own_client,
    });
}
extern "C" fn on_new_channel(server: u64, channel: u64, parent: u64) {
    record(PluginEvent::NewChannel {
        server,
        channel,
        parent,
    });
}
extern "C" fn on_client_move(
    server: u64,
    client: u16,
    old_channel: u64,
    new_channel: u64,
    visibility: c_int,
    message: *const c_char,
) {
    let get_string = slot(|table| table.get_client_variable_as_string);
    let free = slot(|table| table.free_memory);
    let mut nickname: *mut c_char = std::ptr::null_mut();
    let flag = sdk::ClientProperties::Nickname as usize;
    let nickname_seen = (get_string(server, client, flag, &raw mut nickname) == 0).then(|| {
        let seen = text(nickname);
        free(nickname.cast());
        seen
    });
    record(PluginEvent::ClientMove {
        server,
        client,
        old_channel,
        new_channel,
        visibility,
        message: text(message),
        nickname_seen,
    });
}
extern "C" fn on_update_client(
    server: u64,
    client: u16,
    invoker: u16,
    invoker_name: *const c_char,
    _invoker_unique_identifier: *const c_char,
) {
    record(PluginEvent::UpdateClient {
        server,
        client,
        invoker,
        invoker_name: text(invoker_name),
    });
}
extern "C" fn on_plugin_command(
    server: u64,
    plugin_name: *const c_char,
    command: *const c_char,
    invoker: u16,
    invoker_name: *const c_char,
    invoker_unique_identifier: *const c_char,
) {
    record(PluginEvent::PluginCommand {
        server,
        plugin_name: text(plugin_name),
        command: text(command),
        invoker,
        invoker_name: text(invoker_name),
        invoker_unique_identifier: text(invoker_unique_identifier),
    });
}
extern "C" fn on_talk_status_change(
    server: u64,
    status: c_int,
    is_received_whisper: c_int,
    client: u16,
) {
    let get_int = slot(|table| table.get_client_variable_as_int);
    let mut flag_talking: c_int = 0;
    let flag = sdk::ClientProperties::FlagTalking as usize;
    let flag_talking_seen =
        (get_int(server, client, flag, &raw mut flag_talking) == 0).then_some(flag_talking);
    record(PluginEvent::TalkStatus {
        server,
        status,
        is_received_whisper,
        client,
        flag_talking_seen,
    });
}
extern "C" fn on_post_process_voice(
    _server: u64,
    _client: u16,
    samples: *mut i16,
    frames: c_int,
    channels: c_int,
    speakers: *const c_uint,
    fill_mask: *mut c_uint,
) {
    let count = usize::try_from(frames * channels).unwrap();
    // SAFETY: the mock passes `frames * channels` samples and one speaker per channel.
    let (samples, speakers) = unsafe {
        (
            std::slice::from_raw_parts_mut(samples, count),
            std::slice::from_raw_parts(speakers, usize::try_from(channels).unwrap()),
        )
    };
    for sample in samples {
        *sample = sample.saturating_mul(2);
    }
    // SAFETY: the mock passes a writable mask.
    unsafe { fill_mask.write(speakers[0]) };
}
extern "C" fn on_captured_voice(
    _server: u64,
    samples: *mut i16,
    frames: c_int,
    channels: c_int,
    edited: *mut c_int,
) {
    let count = usize::try_from(frames * channels).unwrap();
    // SAFETY: the mock passes `frames * channels` samples and a writable `edited`.
    unsafe {
        for sample in std::slice::from_raw_parts_mut(samples, count) {
            *sample = sample.saturating_neg();
        }
        edited.write(edited.read() | 1);
    }
}
extern "C" fn on_mixed_playback(
    _server: u64,
    samples: *mut i16,
    frames: c_int,
    channels: c_int,
    _speakers: *const c_uint,
    fill_mask: *mut c_uint,
) {
    let count = usize::try_from(frames * channels).unwrap();
    // SAFETY: the mock passes `frames * channels` samples and a writable mask.
    unsafe {
        std::slice::from_raw_parts_mut(samples, count).fill(0);
        fill_mask.write(0);
    }
}
extern "C" fn on_custom_3d_rolloff(_server: u64, _client: u16, distance: f32, volume: *mut f32) {
    // SAFETY: the mock passes a writable volume.
    unsafe { volume.write(volume.read() / (1.0 + distance)) };
}

/// The exports of the in-process plugin, every optional one the drivers use included.
pub(crate) fn in_process_exports() -> PluginExports {
    PluginExports {
        name,
        version,
        api_version,
        author,
        description,
        set_function_pointers,
        init,
        shutdown,
        offers_configure: None,
        register_plugin_id: Some(register_plugin_id),
        command_keyword: Some(command_keyword),
        process_command: Some(process_command),
        current_server_connection_changed: Some(current_server_connection_changed),
        info_title: None,
        info_data: Some(info_data),
        free_memory: Some(free_memory),
        request_autoload: Some(request_autoload),
        on_connect_status_change_event: Some(on_connect_status_change),
        on_new_channel_event: Some(on_new_channel),
        on_new_channel_created_event: None,
        on_del_channel_event: None,
        on_channel_move_event: None,
        on_update_channel_event: None,
        on_update_channel_edited_event: None,
        on_update_client_event: Some(on_update_client),
        on_client_move_event: Some(on_client_move),
        on_client_move_timeout_event: None,
        on_client_move_moved_event: None,
        on_client_kick_from_channel_event: None,
        on_client_kick_from_server_event: None,
        on_client_ban_from_server_event: None,
        on_server_error_event: None,
        on_server_permission_error_event: None,
        on_client_self_variable_update_event: None,
        on_plugin_command_event: Some(on_plugin_command),
        on_talk_status_change_event: Some(on_talk_status_change),
        on_edit_post_process_voice_data_event: Some(on_post_process_voice),
        on_edit_mixed_playback_voice_data_event: Some(on_mixed_playback),
        on_edit_captured_voice_data_event: Some(on_captured_voice),
        on_custom_3d_rolloff_calculation_client_event: Some(on_custom_3d_rolloff),
    }
}

/// A [`MockHost`] over the in-process plugin, holding the plugin's statics for the test.
pub(crate) struct InProcessHost {
    host: MockHost,
    _users: MutexGuard<'static, ()>,
}

impl Deref for InProcessHost {
    type Target = MockHost;
    fn deref(&self) -> &MockHost {
        &self.host
    }
}

impl DerefMut for InProcessHost {
    fn deref_mut(&mut self) -> &mut MockHost {
        &mut self.host
    }
}

/// Starts the in-process plugin with `exports`, `init` answering `init_answer`.
pub(crate) fn try_start(
    exports: &PluginExports,
    init_answer: c_int,
) -> (MutexGuard<'static, ()>, Result<MockHost>) {
    try_start_with(exports, init_answer, &HostOptions::default())
}

/// [`try_start`] with `options`.
pub(crate) fn try_start_with(
    exports: &PluginExports,
    init_answer: c_int,
    options: &HostOptions,
) -> (MutexGuard<'static, ()>, Result<MockHost>) {
    let users = IN_PROCESS_PLUGIN_USERS.lock();
    EVENTS.lock().clear();
    *TABLE.lock() = None;
    FREED_INFO_BLOCKS.store(0, Ordering::SeqCst);
    INIT_ANSWER.store(init_answer, Ordering::SeqCst);
    let library = libloading::Library::from(libloading::os::unix::Library::this());
    let plugin = PluginLibrary::from_parts(PathBuf::from("in_process_plugin"), exports, library);
    (users, MockHost::start(plugin, options))
}

/// Starts the in-process plugin with every export the drivers use.
pub(crate) fn start_in_process_host() -> InProcessHost {
    let (users, host) = try_start(&in_process_exports(), 0);
    InProcessHost {
        host: host.unwrap(),
        _users: users,
    }
}

/// The events since the last take.
pub(crate) fn take_events() -> Vec<PluginEvent> {
    std::mem::take(&mut *EVENTS.lock())
}

/// How many `infoData` blocks came back through `freeMemory`.
pub(crate) fn freed_info_blocks() -> usize {
    FREED_INFO_BLOCKS.load(Ordering::SeqCst)
}
