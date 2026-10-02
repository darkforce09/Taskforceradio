//! The fakes for memory, errors, logging, the chat tab, the host paths and wave playback.
//!
//! **Role:** `freeMemory`, `getErrorMessage`, `logMessage`, `printMessageToCurrentTab`,
//! `getPluginPath`, `getConfigPath` and `playWaveFile`, each with the exact signature of its
//! `Ts3Functions` slot.
//!
//! **Position:** installed into the table by `fake_table`; the plugin calls them.
//!
//! **Signals and state:** each takes the fake-state lock once, briefly; called from any plugin
//! thread (the logging thread calls `logMessage`).
//!
//! **Invariants:** `freeMemory` frees only what the ledger handed out; every other call is
//! logged as a [`HostCall`] before it answers.

use std::ffi::{c_char, c_uint, c_void};
use std::path::PathBuf;

use tfar_ts3_host::{LogLevel, ServerConnectionHandlerId};
use ts3plugin_sys::public_definitions as sdk;

use super::fake_answers::{ERROR_OK, copied_text, hand_out_string, write_path};
use crate::fake_state::host_session::with_fake_teamspeak;
use crate::{HostCall, HostQuery};

/// The text `getErrorMessage` answers for `code`.
pub fn mock_error_text(code: u32) -> String {
    format!("mock TeamSpeak error {code:#06x}")
}

/// `freeMemory`: takes back a block a fake handed out.
pub(crate) extern "C" fn free_memory(pointer: *mut c_void) -> c_uint {
    with_fake_teamspeak(|fake| fake.ledger.free(pointer));
    ERROR_OK
}

/// `getErrorMessage`: a `malloc`ed [`mock_error_text`].
pub(crate) extern "C" fn get_error_message(error_code: c_uint, error: *mut *mut c_char) -> c_uint {
    with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::ErrorMessage(error_code)));
        hand_out_string(
            &mut fake.ledger,
            "getErrorMessage",
            &mock_error_text(error_code),
            error,
        )
    })
}

/// `logMessage`: logged with its level, channel and server.
pub(crate) extern "C" fn log_message(
    log_message: *const c_char,
    severity: sdk::LogLevel,
    channel: *const c_char,
    log_id: u64,
) -> c_uint {
    let call = HostCall::LogMessage {
        text: copied_text(log_message),
        level: log_level(severity),
        channel: copied_text(channel),
        server: ServerConnectionHandlerId(log_id),
    };
    with_fake_teamspeak(|fake| fake.record(call));
    ERROR_OK
}

/// `printMessageToCurrentTab`: logged.
pub(crate) extern "C" fn print_message_to_current_tab(message: *const c_char) {
    let call = HostCall::PrintMessageToCurrentTab {
        text: copied_text(message),
    };
    with_fake_teamspeak(|fake| fake.record(call));
}

/// `getPluginPath(path, maxLen, pluginID)`: the session's plugin folder with a trailing `/`.
pub(crate) extern "C" fn get_plugin_path(
    path: *mut c_char,
    max_len: usize,
    _plugin_id: *const c_char,
) {
    let folder = with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::PluginPath));
        fake.plugin_path.clone()
    });
    write_path(&folder, path, max_len);
}

/// `getConfigPath`: the session's config folder with a trailing `/`.
pub(crate) extern "C" fn get_config_path(path: *mut c_char, max_len: usize) {
    let folder = with_fake_teamspeak(|fake| {
        fake.record(HostCall::Query(HostQuery::ConfigPath));
        fake.config_path.clone()
    });
    write_path(&folder, path, max_len);
}

/// `playWaveFile`: logged; plays nothing.
pub(crate) extern "C" fn play_wave_file(
    server_connection_handler_id: u64,
    path: *const c_char,
) -> c_uint {
    let call = HostCall::PlayWaveFile {
        server: ServerConnectionHandlerId(server_connection_handler_id),
        path: PathBuf::from(copied_text(path)),
    };
    with_fake_teamspeak(|fake| fake.record(call));
    ERROR_OK
}

/// The level of the SDK `severity`.
fn log_level(severity: sdk::LogLevel) -> LogLevel {
    match severity {
        sdk::LogLevel::Critical => LogLevel::Critical,
        sdk::LogLevel::Error => LogLevel::Error,
        sdk::LogLevel::Warning => LogLevel::Warning,
        sdk::LogLevel::Debug => LogLevel::Debug,
        sdk::LogLevel::Info => LogLevel::Info,
        sdk::LogLevel::Devel => LogLevel::Devel,
    }
}
