//! Unit tests for `ts3_plugin!`: the macro is expanded on a dummy plugin here, and the generated
//! `extern "C"` exports are called through function pointers the way TeamSpeak calls them, with a
//! fake function table installed process-wide.
//!
//! The dummy plugin is one `static` shared by every test, so each test uses its own server
//! connection ID and reads back only what the plugin saw for that ID.

use std::ffi::{CStr, c_char, c_int, c_uint, c_void};
use std::sync::{Mutex, Once};
use std::thread::ThreadId;

use ts3plugin_sys::public_definitions::LogLevel as SdkLogLevel;
use ts3plugin_sys::ts3functions::Ts3Functions;

use crate::fake_ts3_table::fake_table;
use crate::{
    CapturedVoiceData, ClientMove, CommandHandling, ErrorDisplay, InfoItem, InitStatus, Invoker,
    MixedPlaybackVoiceData, PluginCommand, PostProcessVoiceData, ServerConnectionHandlerId,
    ServerError, Ts3, Ts3Plugin,
};

/// The plugin under test: records every call as `(server, description)`.
struct DummyPlugin {
    seen: Mutex<Vec<(u64, String)>>,
}

static PLUGIN: DummyPlugin = DummyPlugin {
    seen: Mutex::new(Vec::new()),
};

crate::ts3_plugin!(PLUGIN);

impl DummyPlugin {
    fn record(&self, server: u64, description: String) {
        self.seen.lock().unwrap().push((server, description));
    }
}

fn seen_for(server: u64) -> Vec<String> {
    PLUGIN
        .seen
        .lock()
        .unwrap()
        .iter()
        .filter(|(id, _)| *id == server)
        .map(|(_, text)| text.clone())
        .collect()
}

impl Ts3Plugin for DummyPlugin {
    const NAME: &'static CStr = c"Dummy Plugin";
    const VERSION: &'static CStr = c"9.9.9";
    const AUTHOR: &'static CStr = c"Tester";
    const DESCRIPTION: &'static CStr = c"A plugin for the tests";
    const COMMAND_KEYWORD: &'static CStr = c"dummy";

    fn init(&self, ts3: &'static Ts3) -> InitStatus {
        self.record(0, format!("init id={:?}", ts3.plugin_id()));
        InitStatus::Success
    }

    fn register_plugin_id(&self, plugin_id: &str) {
        self.record(0, format!("register {plugin_id}"));
    }

    fn process_command(&self, server: ServerConnectionHandlerId, command: &str) -> CommandHandling {
        self.record(server.0, format!("command {command}"));
        if command == "diag" {
            CommandHandling::Handled
        } else {
            CommandHandling::NotHandled
        }
    }

    fn info_data(&self, server: ServerConnectionHandlerId, item: InfoItem) -> Option<String> {
        self.record(server.0, format!("info {item:?}"));
        match item {
            InfoItem::Client(client) => Some(format!("<TFAR>client {}</TFAR>", client.0)),
            _ => None,
        }
    }

    fn on_client_move_moved(&self, movement: ClientMove<'_>, mover: Invoker<'_>) {
        self.record(
            movement.server.0,
            format!(
                "moved {} {}->{} {:?} '{}' by {} '{}' '{}'",
                movement.client.0,
                movement.old_channel.0,
                movement.new_channel.0,
                movement.visibility,
                movement.message,
                mover.id.0,
                mover.name,
                mover.unique_identifier
            ),
        );
    }

    fn on_client_ban_from_server(
        &self,
        movement: ClientMove<'_>,
        kicker: Invoker<'_>,
        ban_seconds: u64,
    ) {
        self.record(
            movement.server.0,
            format!(
                "ban {} by '{}' for {ban_seconds}",
                movement.client.0, kicker.name
            ),
        );
    }

    fn on_plugin_command(&self, command: PluginCommand<'_>) {
        self.record(
            command.server.0,
            format!(
                "plugin {} '{}' from {} '{}' '{}'",
                command.plugin_name,
                command.command,
                command.invoker.id.0,
                command.invoker.name,
                command.invoker.unique_identifier
            ),
        );
    }

    fn on_server_error(&self, error: ServerError<'_>) -> ErrorDisplay {
        self.record(
            error.server.0,
            format!(
                "error {} '{}' '{}'",
                error.error, error.message, error.return_code
            ),
        );
        ErrorDisplay::Suppress
    }

    fn on_connect_status_change(
        &self,
        server: ServerConnectionHandlerId,
        status: crate::ConnectStatus,
        error: u32,
    ) {
        self.record(server.0, format!("status {status:?} {error}"));
    }

    fn on_talk_status_change(
        &self,
        server: ServerConnectionHandlerId,
        status: crate::TalkStatus,
        is_received_whisper: bool,
        client: crate::ClientId,
    ) {
        self.record(
            server.0,
            format!(
                "talk {} {status:?} talking={} whisper={is_received_whisper}",
                client.0,
                status.is_talking()
            ),
        );
    }

    fn on_edit_post_process_voice_data(&self, data: PostProcessVoiceData<'_>) {
        self.record(
            data.server.0,
            format!(
                "post {} len={} channels={} frames={} speakers={:?} mask={}",
                data.client.0,
                data.samples.len(),
                data.channels,
                data.frames,
                data.channel_speakers,
                data.channel_fill_mask
            ),
        );
        for (index, sample) in data.samples.iter_mut().enumerate() {
            *sample = i16::try_from(index).unwrap() * 10;
        }
        *data.channel_fill_mask |= 3;
    }

    fn on_edit_mixed_playback_voice_data(&self, data: MixedPlaybackVoiceData<'_>) {
        self.record(
            data.server.0,
            format!(
                "mixed len={} channels={} frames={} mask={}",
                data.samples.len(),
                data.channels,
                data.frames,
                data.channel_fill_mask
            ),
        );
        data.samples.fill(1);
        *data.channel_fill_mask |= 3;
    }

    fn on_edit_captured_voice_data(&self, data: CapturedVoiceData<'_>) {
        self.record(
            data.server.0,
            format!(
                "captured len={} channels={} frames={} edited={}",
                data.samples.len(),
                data.channels,
                data.frames,
                data.edited
            ),
        );
        data.samples
            .iter_mut()
            .for_each(|sample| *sample = -*sample);
        *data.edited |= 1;
    }

    fn on_custom_3d_rolloff_calculation_client(
        &self,
        server: ServerConnectionHandlerId,
        client: crate::ClientId,
        distance: f32,
        volume: &mut f32,
    ) {
        self.record(
            server.0,
            format!("rolloff {} {distance} {volume}", client.0),
        );
        *volume = 1.0;
    }
}

/// What the fake `printMessageToCurrentTab` and `logMessage` received, process-wide.
static PRINTED: Mutex<Vec<String>> = Mutex::new(Vec::new());
static LOGGED: Mutex<Vec<(ThreadId, String, String)>> = Mutex::new(Vec::new());

extern "C" fn fake_print(message: *const c_char) {
    // SAFETY: the code under test passes a NUL-terminated message valid for the call.
    let message = unsafe { CStr::from_ptr(message) }
        .to_string_lossy()
        .into_owned();
    PRINTED.lock().unwrap().push(message);
}

extern "C" fn fake_log(
    text: *const c_char,
    _level: SdkLogLevel,
    channel: *const c_char,
    _log_id: u64,
) -> c_uint {
    // SAFETY: the code under test passes NUL-terminated strings valid for the call.
    let (text, channel) = unsafe { (CStr::from_ptr(text), CStr::from_ptr(channel)) };
    LOGGED.lock().unwrap().push((
        std::thread::current().id(),
        text.to_string_lossy().into_owned(),
        channel.to_string_lossy().into_owned(),
    ));
    0
}

/// Installs the fake table and the plugin ID the way TeamSpeak does at load, once per process.
fn load_like_teamspeak() {
    static LOAD: Once = Once::new();
    LOAD.call_once(|| {
        let mut table: Ts3Functions = fake_table();
        table.print_message_to_current_tab = fake_print;
        table.log_message = fake_log;
        let set_function_pointers: extern "C" fn(Ts3Functions) = ts3plugin_setFunctionPointers;
        set_function_pointers(table);
        let register: extern "C" fn(*const c_char) = ts3plugin_registerPluginID;
        register(c"dummy_plugin_id".as_ptr());
    });
}

fn read(text: *const c_char) -> String {
    assert!(!text.is_null());
    // SAFETY: the exports return pointers to `'static` NUL-terminated constants.
    unsafe { CStr::from_ptr(text) }.to_str().unwrap().to_owned()
}

#[test]
fn the_metadata_exports_return_the_plugin_constants() {
    let exports: [(extern "C" fn() -> *const c_char, &str); 6] = [
        (ts3plugin_name, "Dummy Plugin"),
        (ts3plugin_version, "9.9.9"),
        (ts3plugin_author, "Tester"),
        (ts3plugin_description, "A plugin for the tests"),
        (ts3plugin_infoTitle, "Dummy Plugin"),
        (ts3plugin_commandKeyword, "dummy"),
    ];
    for (export, expected) in exports {
        assert_eq!(read(export()), expected);
    }
    let fixed: [(extern "C" fn() -> c_int, c_int); 3] = [
        (ts3plugin_apiVersion, 26),
        (ts3plugin_offersConfigure, 0),
        (ts3plugin_requestAutoload, 1),
    ];
    for (export, expected) in fixed {
        assert_eq!(export(), expected);
    }
}

#[test]
fn init_sees_the_installed_table_and_the_registered_id() {
    load_like_teamspeak();
    let init: extern "C" fn() -> c_int = ts3plugin_init;
    assert_eq!(init(), 0);
    let seen = seen_for(0);
    assert!(
        seen.contains(&"register dummy_plugin_id".to_owned()),
        "{seen:?}"
    );
    assert!(
        seen.contains(&"init id=Some(\"dummy_plugin_id\")".to_owned()),
        "{seen:?}"
    );
    assert_eq!(
        Ts3::installed().unwrap().plugin_id().as_deref(),
        Some(&c"dummy_plugin_id".to_owned())
    );
    let shutdown: extern "C" fn() = ts3plugin_shutdown;
    shutdown();
}

#[test]
fn process_command_passes_the_text_and_returns_the_plugin_answer() {
    let process: extern "C" fn(u64, *const c_char) -> c_int = ts3plugin_processCommand;
    assert_eq!(process(101, c"diag".as_ptr()), 0);
    assert_eq!(process(101, c"nonsense".as_ptr()), 1);
    assert_eq!(process(101, std::ptr::null()), 1);
    assert_eq!(
        seen_for(101),
        ["command diag", "command nonsense", "command "]
    );
}

#[test]
fn info_data_hands_out_malloc_text_that_free_memory_releases() {
    let info: extern "C" fn(u64, u64, c_int, *mut *mut c_char) = ts3plugin_infoData;
    let free: extern "C" fn(*mut c_void) = ts3plugin_freeMemory;
    let mut data: *mut c_char = std::ptr::null_mut();
    info(102, 7, 2, &raw mut data);
    assert_eq!(read(data), "<TFAR>client 7</TFAR>");
    free(data.cast());

    let mut data: *mut c_char = c"stale".as_ptr().cast_mut();
    info(102, 3, 1, &raw mut data);
    assert!(
        data.is_null(),
        "a channel gets no text, and the slot is cleared"
    );
    info(102, 3, 9, &raw mut data);
    assert!(data.is_null(), "an unknown item type gets no text");
    info(102, 3, 2, std::ptr::null_mut());
    free(std::ptr::null_mut());
    assert_eq!(
        seen_for(102),
        ["info Client(ClientId(7))", "info Channel(ChannelId(3))"]
    );
}

#[test]
fn a_moved_client_arrives_with_newtypes_text_and_the_mover() {
    let moved: extern "C" fn(
        u64,
        u16,
        u64,
        u64,
        c_int,
        u16,
        *const c_char,
        *const c_char,
        *const c_char,
    ) = ts3plugin_onClientMoveMovedEvent;
    moved(
        103,
        12,
        4,
        5,
        2,
        1,
        c"Admin".as_ptr(),
        c"uid=".as_ptr(),
        c"go".as_ptr(),
    );
    moved(
        103,
        13,
        0,
        6,
        7,
        2,
        std::ptr::null(),
        std::ptr::null(),
        std::ptr::null(),
    );
    assert_eq!(
        seen_for(103),
        [
            "moved 12 4->5 Leave 'go' by 1 'Admin' 'uid='",
            "moved 13 0->6 Other(7) '' by 2 '' ''"
        ]
    );
}

#[test]
fn a_ban_carries_its_length() {
    let ban: extern "C" fn(
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
    ) = ts3plugin_onClientBanFromServerEvent;
    ban(
        104,
        20,
        1,
        0,
        2,
        3,
        c"Mod".as_ptr(),
        c"m=".as_ptr(),
        600,
        c"bye".as_ptr(),
    );
    assert_eq!(seen_for(104), ["ban 20 by 'Mod' for 600"]);
}

#[test]
fn a_plugin_command_has_the_api_26_invoker_arguments() {
    let command: extern "C" fn(
        u64,
        *const c_char,
        *const c_char,
        u16,
        *const c_char,
        *const c_char,
    ) = ts3plugin_onPluginCommandEvent;
    command(
        105,
        c"TFAR_1_0".as_ptr(),
        c"VOLUME\tnick\t20\ttrue".as_ptr(),
        44,
        c"Nick".as_ptr(),
        c"abc=".as_ptr(),
    );
    assert_eq!(
        seen_for(105),
        ["plugin TFAR_1_0 'VOLUME\tnick\t20\ttrue' from 44 'Nick' 'abc='"]
    );
}

#[test]
fn a_server_error_returns_the_plugin_choice_and_status_reads_the_enum() {
    let error: extern "C" fn(u64, *const c_char, c_uint, *const c_char, *const c_char) -> c_int =
        ts3plugin_onServerErrorEvent;
    assert_eq!(
        error(
            106,
            c"denied".as_ptr(),
            0x0a08,
            c"rc".as_ptr(),
            std::ptr::null()
        ),
        1
    );
    let status: extern "C" fn(u64, c_int, c_uint) = ts3plugin_onConnectStatusChangeEvent;
    status(106, 4, 0);
    assert_eq!(
        seen_for(106),
        ["error 2568 'denied' 'rc'", "status ConnectionEstablished 0"]
    );
}

#[test]
fn post_process_voice_gets_a_slice_bounded_by_the_counts() {
    let post: extern "C" fn(u64, u16, *mut i16, c_int, c_int, *const c_uint, *mut c_uint) =
        ts3plugin_onEditPostProcessVoiceDataEvent;
    // Two channels of four frames, with a guard sample after the block that must stay untouched.
    let mut buffer = [5_i16; 9];
    let speakers = [1_u32, 2];
    let mut mask: c_uint = 1;
    post(
        107,
        9,
        buffer.as_mut_ptr(),
        4,
        2,
        speakers.as_ptr(),
        &raw mut mask,
    );
    assert_eq!(buffer, [0, 10, 20, 30, 40, 50, 60, 70, 5]);
    assert_eq!(mask, 3);
    assert_eq!(
        seen_for(107),
        ["post 9 len=8 channels=2 frames=4 speakers=[1, 2] mask=1"]
    );
}

#[test]
fn null_or_negative_voice_arguments_become_empty_slices() {
    let post: extern "C" fn(u64, u16, *mut i16, c_int, c_int, *const c_uint, *mut c_uint) =
        ts3plugin_onEditPostProcessVoiceDataEvent;
    post(
        108,
        1,
        std::ptr::null_mut(),
        480,
        2,
        std::ptr::null(),
        std::ptr::null_mut(),
    );
    let mut buffer = [7_i16; 4];
    post(
        108,
        1,
        buffer.as_mut_ptr(),
        -1,
        2,
        std::ptr::null(),
        std::ptr::null_mut(),
    );
    post(
        108,
        1,
        buffer.as_mut_ptr(),
        2,
        -2,
        std::ptr::null(),
        std::ptr::null_mut(),
    );
    assert_eq!(buffer, [7; 4]);
    assert_eq!(
        seen_for(108),
        [
            "post 1 len=0 channels=2 frames=0 speakers=[] mask=0",
            "post 1 len=0 channels=2 frames=0 speakers=[] mask=0",
            "post 1 len=0 channels=0 frames=0 speakers=[] mask=0",
        ]
    );
}

#[test]
fn mixed_playback_and_capture_write_back_into_teamspeaks_buffers() {
    let mixed: extern "C" fn(u64, *mut i16, c_int, c_int, *const c_uint, *mut c_uint) =
        ts3plugin_onEditMixedPlaybackVoiceDataEvent;
    let mut playback = [0_i16; 6];
    let mut mask: c_uint = 0;
    mixed(
        109,
        playback.as_mut_ptr(),
        3,
        2,
        std::ptr::null(),
        &raw mut mask,
    );
    assert_eq!((playback, mask), ([1; 6], 3));

    let captured: extern "C" fn(u64, *mut i16, c_int, c_int, *mut c_int) =
        ts3plugin_onEditCapturedVoiceDataEvent;
    let mut microphone = [3_i16, -4, 5];
    let mut edited: c_int = 2;
    captured(109, microphone.as_mut_ptr(), 3, 1, &raw mut edited);
    assert_eq!((microphone, edited), ([-3, 4, -5], 3));
    captured(109, std::ptr::null_mut(), 3, 1, std::ptr::null_mut());
    assert_eq!(
        seen_for(109),
        [
            "mixed len=6 channels=2 frames=3 mask=0",
            "captured len=3 channels=1 frames=3 edited=2",
            "captured len=0 channels=1 frames=0 edited=0",
        ]
    );
}

#[test]
fn the_rolloff_volume_is_written_through_and_null_is_ignored() {
    let rolloff: extern "C" fn(u64, u16, f32, *mut f32) =
        ts3plugin_onCustom3dRolloffCalculationClientEvent;
    let mut volume = 0.5_f32;
    rolloff(110, 3, 12.5, &raw mut volume);
    assert!((volume - 1.0).abs() < f32::EPSILON);
    rolloff(110, 3, 1.0, std::ptr::null_mut());
    assert_eq!(seen_for(110), ["rolloff 3 12.5 0.5", "rolloff 3 1 1"]);
}

#[test]
fn print_to_current_tab_and_install_logging_use_the_installed_table() {
    load_like_teamspeak();
    crate::print_to_current_tab("[B]TFAR[/B] ready").unwrap();
    assert!(
        PRINTED
            .lock()
            .unwrap()
            .contains(&"[B]TFAR[/B] ready".to_owned())
    );

    let guard = crate::install_logging("Dummy Plugin").unwrap();
    let emitter = std::thread::current().id();
    tracing::info!(clients = 3, "logged through the global subscriber");
    drop(guard);
    let logged = LOGGED.lock().unwrap();
    let line = logged
        .iter()
        .find(|(_, text, _)| text.contains("logged through the global subscriber"))
        .expect("the line reached logMessage");
    assert_ne!(line.0, emitter, "logMessage ran on the emitting thread");
    assert_eq!(line.2, "Dummy Plugin");
    assert!(
        line.1
            .ends_with("logged through the global subscriber clients=3"),
        "{}",
        line.1
    );
}

#[test]
fn a_talk_status_change_arrives_with_the_status_the_whisper_flag_and_the_client() {
    let talk: extern "C" fn(u64, c_int, c_int, u16) = ts3plugin_onTalkStatusChangeEvent;
    talk(111, 1, 0, 7);
    talk(111, 0, 0, 7);
    talk(111, 1, 1, 8);
    talk(111, 2, 0, 1);
    talk(111, 9, 3, 9);
    assert_eq!(
        seen_for(111),
        [
            "talk 7 Talking talking=true whisper=false",
            "talk 7 NotTalking talking=false whisper=false",
            "talk 8 Talking talking=true whisper=true",
            "talk 1 TalkingWhileDisabled talking=false whisper=false",
            "talk 9 Other(9) talking=false whisper=true",
        ]
    );
}
