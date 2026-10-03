//! The bodies of the lifecycle and metadata exports: name and version strings, the function
//! table, init and shutdown, the plugin ID, chat commands and the info panel.
//!
//! **Role:** turns each raw `ts3plugin_*` call the macro forwards into a [`Ts3Plugin`] call and
//! the answer back into the integer or pointer TeamSpeak expects.
//!
//! **Position:** called only from the expansion of [`ts3_plugin!`](crate::ts3_plugin); public but
//! hidden, since a macro in another crate can reach only public items.
//!
//! **Signals and state:** all of these run on TeamSpeak's main thread. `set_function_pointers`
//! writes the process-wide [`Ts3`]; `register_plugin_id` replaces its plugin ID with each new one.
//!
//! **Invariants:** nothing here panics; a missing function table makes `init` fail with
//! [`InitStatus::Failure`] instead of calling the plugin.

use std::ffi::{CStr, c_char, c_int, c_void};

use ts3plugin_sys::ts3functions::Ts3Functions;

use super::plugin_answers::{InfoItem, InitStatus};
use super::plugin_memory::{free_c_block, malloc_c_string};
use super::plugin_trait::Ts3Plugin;
use crate::ts3_functions::owned_memory::borrowed_text;
use crate::{ServerConnectionHandlerId, Ts3};

/// The plugin API version both plugins report: TeamSpeak 3.5 and 3.6 (`ts3plugin_apiVersion`).
pub const API_VERSION: c_int = 26;

/// `PLUGIN_OFFERS_NO_CONFIGURE`: neither plugin has a configure dialog.
pub const OFFERS_CONFIGURE: c_int = 0;

/// `ts3plugin_requestAutoload`: `1` asks TeamSpeak to load the plugin at start.
pub const REQUESTS_AUTOLOAD: c_int = 1;

/// Which metadata string TeamSpeak asks for.
#[derive(Clone, Copy)]
pub enum MetadataText {
    /// `ts3plugin_name`.
    Name,
    /// `ts3plugin_version`.
    Version,
    /// `ts3plugin_author`.
    Author,
    /// `ts3plugin_description`.
    Description,
    /// `ts3plugin_infoTitle`.
    InfoTitle,
    /// `ts3plugin_commandKeyword`.
    CommandKeyword,
}

/// The pointer TeamSpeak reads a metadata string from: one of the plugin's `'static` `CStr`
/// constants, so it never dangles.
pub fn metadata_text<P: Ts3Plugin>(_plugin: &'static P, which: MetadataText) -> *const c_char {
    let text: &'static CStr = match which {
        MetadataText::Name => P::NAME,
        MetadataText::Version => P::VERSION,
        MetadataText::Author => P::AUTHOR,
        MetadataText::Description => P::DESCRIPTION,
        MetadataText::InfoTitle => P::INFO_TITLE,
        MetadataText::CommandKeyword => P::COMMAND_KEYWORD,
    };
    text.as_ptr()
}

/// `ts3plugin_setFunctionPointers`: installs TeamSpeak's table process-wide.
pub fn set_function_pointers(functions: Ts3Functions) {
    let _ = Ts3::install(functions);
}

/// `ts3plugin_init`: runs the plugin's `init`, or fails when no table is installed.
pub fn init<P: Ts3Plugin>(plugin: &'static P) -> c_int {
    match Ts3::installed() {
        Some(ts3) => plugin.init(ts3).to_raw(),
        None => InitStatus::Failure.to_raw(),
    }
}

/// `ts3plugin_shutdown`.
pub fn shutdown<P: Ts3Plugin>(plugin: &'static P) {
    plugin.shutdown();
}

/// `ts3plugin_registerPluginID`: stores the ID in [`Ts3`], replacing the one an earlier load
/// registered, and tells the plugin.
///
/// # Safety
///
/// `plugin_id` is null or a NUL-terminated string valid for this call, as TeamSpeak passes it.
pub unsafe fn register_plugin_id<P: Ts3Plugin>(plugin: &'static P, plugin_id: *const c_char) {
    if plugin_id.is_null() {
        return;
    }
    // SAFETY: non-null and NUL-terminated for the call by the caller's contract; the ID is copied
    // before the call returns.
    let id = unsafe { CStr::from_ptr(plugin_id) };
    if let Some(ts3) = Ts3::installed() {
        ts3.register_plugin_id(id);
    }
    plugin.register_plugin_id(&id.to_string_lossy());
}

/// `ts3plugin_processCommand`.
///
/// # Safety
///
/// `command` is null or a NUL-terminated string valid for this call.
pub unsafe fn process_command<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    command: *const c_char,
) -> c_int {
    // SAFETY: null or NUL-terminated for the call, by the caller's contract.
    let command = unsafe { borrowed_text(command) };
    plugin
        .process_command(ServerConnectionHandlerId(server), &command)
        .to_raw()
}

/// `ts3plugin_currentServerConnectionChanged`.
pub fn current_server_connection_changed<P: Ts3Plugin>(plugin: &'static P, server: u64) {
    plugin.current_server_connection_changed(ServerConnectionHandlerId(server));
}

/// `ts3plugin_infoData`: writes a `malloc`ed copy of the plugin's text into `*data`, or null for
/// none; TeamSpeak hands the block back to `ts3plugin_freeMemory`.
///
/// # Safety
///
/// `data` is null or points at a writable `char*` slot, as TeamSpeak passes it.
pub unsafe fn info_data<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    id: u64,
    item_type: c_int,
    data: *mut *mut c_char,
) {
    if data.is_null() {
        return;
    }
    let text = InfoItem::from_raw(item_type, id)
        .and_then(|item| plugin.info_data(ServerConnectionHandlerId(server), item));
    let block = text.map_or(std::ptr::null_mut(), |text| malloc_c_string(&text));
    // SAFETY: `data` is non-null and writable by the caller's contract; the block written is
    // owned by TeamSpeak until it calls `ts3plugin_freeMemory` with it.
    unsafe { data.write(block) };
}

/// `ts3plugin_freeMemory`: releases a block the plugin handed TeamSpeak.
///
/// # Safety
///
/// `block` is null or a pointer `info_data` wrote, not freed before.
pub unsafe fn free_memory(block: *mut c_void) {
    // SAFETY: forwarded unchanged under the same contract.
    unsafe { free_c_block(block) }
}
