//! The helpers every fake uses to read the plugin's arguments and write its answers, and the SDK
//! error codes the fakes return.
//!
//! **Role:** the one place that touches plugin memory: C strings and zero-terminated ID arrays
//! in, answer slots, `malloc`ed strings and lists and path buffers out.
//!
//! **Position:** under the fake files of `fake_functions`; the allocations go through the ledger
//! of the fake state.
//!
//! **Signals and state:** no state of its own; runs on whichever thread the plugin calls from,
//! with the fake-state lock held by the caller where a ledger is passed.
//!
//! **Invariants:** every pointer is checked for null before use; a path buffer gets at most
//! `max_len - 1` bytes and a NUL; a list handed out ends in its `0` terminator.

use std::ffi::{CStr, c_char, c_uint};
use std::path::Path;

use tfar_ts3_host::{ClientId, Ts3Vector};
use ts3plugin_sys::public_definitions as sdk;
use ts3plugin_sys::public_errors::Error as SdkError;

use crate::fake_state::allocation_ledger::AllocationLedger;

/// `ERROR_ok`.
pub(crate) const ERROR_OK: c_uint = SdkError::Ok as c_uint;
/// `ERROR_not_implemented`: the answer of a slot no fake implements.
pub(crate) const ERROR_NOT_IMPLEMENTED: c_uint = SdkError::NotImplemented as c_uint;
/// `ERROR_parameter_invalid`: a null answer slot, an unknown flag or setting.
pub(crate) const ERROR_PARAMETER_INVALID: c_uint = SdkError::ParameterInvalid as c_uint;
/// `ERROR_not_connected`: an unknown server connection handler.
pub(crate) const ERROR_NOT_CONNECTED: c_uint = SdkError::NotConnected as c_uint;
/// `ERROR_client_invalid_id`: a client the server does not have.
pub(crate) const ERROR_CLIENT_INVALID_ID: c_uint = SdkError::ClientInvalidId as c_uint;
/// `ERROR_channel_invalid_id`: a channel the server does not have.
pub(crate) const ERROR_CHANNEL_INVALID_ID: c_uint = SdkError::ChannelInvalidId as c_uint;
/// `ERROR_out_of_memory`: `malloc` failed.
pub(crate) const ERROR_OUT_OF_MEMORY: c_uint = SdkError::OutOfMemory as c_uint;

/// The text of the C string at `text`, lossily as UTF-8; null reads as `""`.
pub(crate) fn copied_text(text: *const c_char) -> String {
    if text.is_null() {
        return String::new();
    }
    // SAFETY: the SDK passes every string argument NUL-terminated and valid for the call; it is
    // copied before the fake returns.
    unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned()
}

/// Writes `value` into the plugin's answer slot; `ERROR_parameter_invalid` for a null slot.
pub(crate) fn write_answer<Value>(slot: *mut Value, value: Value) -> c_uint {
    if slot.is_null() {
        return ERROR_PARAMETER_INVALID;
    }
    // SAFETY: the SDK makes every answer slot a writable, aligned `Value` the plugin owns for the
    // call; it is non-null here.
    unsafe { slot.write(value) };
    ERROR_OK
}

/// Hands the plugin a `malloc`ed copy of `text`, recorded for `function`, through `slot`.
pub(crate) fn hand_out_string(
    ledger: &mut AllocationLedger,
    function: &'static str,
    text: &str,
    slot: *mut *mut c_char,
) -> c_uint {
    if slot.is_null() {
        return ERROR_PARAMETER_INVALID;
    }
    let mut bytes = text.as_bytes().to_vec();
    // An interior NUL ends the C string early, as TeamSpeak's own strings would.
    bytes.push(0);
    let block = ledger.hand_out(function, &bytes);
    if block.is_null() {
        return ERROR_OUT_OF_MEMORY;
    }
    write_answer(slot, block.cast::<c_char>())
}

/// Hands the plugin a `malloc`ed copy of `items` with a `0` terminator, recorded for `function`,
/// through `slot`.
pub(crate) fn hand_out_list<Item: Copy + Default>(
    ledger: &mut AllocationLedger,
    function: &'static str,
    items: &[Item],
    slot: *mut *mut Item,
) -> c_uint {
    if slot.is_null() {
        return ERROR_PARAMETER_INVALID;
    }
    let terminated: Vec<Item> = items
        .iter()
        .copied()
        .chain(std::iter::once(Item::default()))
        .collect();
    let byte_count = size_of_val(terminated.as_slice());
    // SAFETY: `terminated` is `byte_count` initialised bytes of plain `Copy` integers.
    let bytes = unsafe { std::slice::from_raw_parts(terminated.as_ptr().cast::<u8>(), byte_count) };
    let block = ledger.hand_out(function, bytes);
    if block.is_null() {
        return ERROR_OUT_OF_MEMORY;
    }
    write_answer(slot, block.cast::<Item>())
}

/// The clients of a zero-terminated `anyID` array; null reads as empty.
pub(crate) fn terminated_clients(clients: *const u16) -> Vec<ClientId> {
    let mut read = Vec::new();
    if clients.is_null() {
        return read;
    }
    let mut cursor = clients;
    loop {
        // SAFETY: the SDK requires the array to end in `0`; the loop stops on it, so every read
        // is inside the plugin's array.
        let client = unsafe { cursor.read() };
        if client == 0 {
            return read;
        }
        read.push(ClientId(client));
        // SAFETY: the element just read was not the terminator, so the next one is in the array.
        cursor = unsafe { cursor.add(1) };
    }
}

/// The vector at `vector`; null reads as `None`.
pub(crate) fn read_vector(vector: *const sdk::Ts3Vector) -> Option<Ts3Vector> {
    if vector.is_null() {
        return None;
    }
    // SAFETY: a non-null `TS3_VECTOR*` argument points at a readable vector for the call.
    let sdk::Ts3Vector { x, y, z } = unsafe { vector.read() };
    Some(Ts3Vector { x, y, z })
}

/// Writes `folder` with a trailing `/` into the plugin's `buffer` of `max_len` bytes, truncated
/// to `max_len - 1` bytes and NUL-terminated, as TeamSpeak's path functions do.
pub(crate) fn write_path(folder: &Path, buffer: *mut c_char, max_len: usize) {
    if buffer.is_null() || max_len == 0 {
        return;
    }
    let mut bytes = folder.as_os_str().as_encoded_bytes().to_vec();
    if !bytes.ends_with(b"/") {
        bytes.push(b'/');
    }
    let length = bytes.len().min(max_len.saturating_sub(1));
    let kept = bytes.get(..length).unwrap_or_default();
    // SAFETY: the plugin owns `max_len` writable bytes at `buffer`; `length + 1 <= max_len`.
    unsafe {
        std::ptr::copy_nonoverlapping(kept.as_ptr(), buffer.cast::<u8>(), length);
        buffer.add(length).write(0);
    }
}

/// Writes an empty path (a lone NUL) into the plugin's `buffer` of `max_len` bytes, as TeamSpeak
/// answers `getPluginPath` for a plugin ID it does not know.
pub(crate) fn write_empty_path(buffer: *mut c_char, max_len: usize) {
    if buffer.is_null() || max_len == 0 {
        return;
    }
    // SAFETY: the plugin owns `max_len >= 1` writable bytes at `buffer`.
    unsafe { buffer.write(0) };
}
