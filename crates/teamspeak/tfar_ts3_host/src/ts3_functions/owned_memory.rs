//! Copying memory TeamSpeak owns into Rust values: strings it allocates, zero-terminated ID lists,
//! strings it lends to a callback, and the path buffers it fills.
//!
//! **Role:** the one place that turns TeamSpeak pointers into owned Rust data and hands
//! TeamSpeak's allocations back to its `freeMemory`.
//!
//! **Position:** inside `ts3_functions`; used by the method files and by `plugin_exports` for the
//! string arguments of callbacks.
//!
//! **Signals and state:** no state; runs on whatever thread makes the call.
//!
//! **Invariants:** every pointer is checked for null before it is read; every TeamSpeak allocation
//! taken here is freed exactly once, after its contents are copied.

use std::borrow::Cow;
use std::ffi::{CStr, c_char, c_void};
use std::path::PathBuf;

use super::handle::Ts3;

impl Ts3 {
    /// Copies the NUL-terminated string at `text` (lossy UTF-8) and frees it with `freeMemory`.
    ///
    /// A null `text` gives an empty string and frees nothing.
    ///
    /// # Safety
    ///
    /// `text` is null or points at a NUL-terminated string TeamSpeak allocated and handed to the
    /// caller, which passes its ownership here; nothing reads `text` afterwards.
    pub(crate) unsafe fn take_string(&self, text: *mut c_char) -> String {
        if text.is_null() {
            return String::new();
        }
        // SAFETY: `text` is non-null and NUL-terminated by the caller's contract, and stays valid
        // until the `freeMemory` call below.
        let copied = unsafe { CStr::from_ptr(text) }
            .to_string_lossy()
            .into_owned();
        (self.functions.free_memory)(text.cast::<c_void>());
        copied
    }

    /// Copies the zero-terminated list at `list` and frees it with `freeMemory`.
    ///
    /// A null `list` gives an empty vector and frees nothing.
    ///
    /// # Safety
    ///
    /// `list` is null or points at an array TeamSpeak allocated and handed to the caller, ending in
    /// an element equal to `T::default()` (TeamSpeak's `0` terminator); ownership passes here.
    pub(crate) unsafe fn take_terminated_list<T>(&self, list: *mut T) -> Vec<T>
    where
        T: Copy + Default + PartialEq,
    {
        if list.is_null() {
            return Vec::new();
        }
        let terminator = T::default();
        let mut items = Vec::new();
        let mut cursor = list.cast_const();
        loop {
            // SAFETY: `cursor` starts at the non-null list and advances one element per item read;
            // the caller guarantees a terminator ends the array, and the loop stops on it, so every
            // read is inside the allocation.
            let item = unsafe { cursor.read() };
            if item == terminator {
                break;
            }
            items.push(item);
            // SAFETY: the element just read was not the terminator, so the next element still lies
            // inside the same allocation.
            cursor = unsafe { cursor.add(1) };
        }
        (self.functions.free_memory)(list.cast::<c_void>());
        items
    }
}

/// Reads a string TeamSpeak lends to a callback, lossily as UTF-8; null reads as `""`.
///
/// # Safety
///
/// `text` is null or points at a NUL-terminated string that stays valid and unchanged for `'text`;
/// TeamSpeak's callback arguments are valid for the duration of the callback.
pub(crate) unsafe fn borrowed_text<'text>(text: *const c_char) -> Cow<'text, str> {
    if text.is_null() {
        return Cow::Borrowed("");
    }
    // SAFETY: non-null and NUL-terminated for `'text` by the caller's contract.
    unsafe { CStr::from_ptr(text) }.to_string_lossy()
}

/// The text of a path buffer TeamSpeak filled: the bytes before the first NUL, or the whole buffer
/// when TeamSpeak wrote no NUL.
pub(crate) fn path_from_buffer(buffer: &[u8]) -> PathBuf {
    let length = buffer
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(buffer.len());
    let bytes = buffer.get(..length).unwrap_or_default();
    path_from_bytes(bytes)
}

/// A path from the bytes TeamSpeak wrote: raw bytes on Unix, lossy UTF-8 elsewhere.
#[cfg(unix)]
fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    use std::os::unix::ffi::OsStrExt;
    PathBuf::from(std::ffi::OsStr::from_bytes(bytes))
}

/// A path from the bytes TeamSpeak wrote: raw bytes on Unix, lossy UTF-8 elsewhere.
#[cfg(not(unix))]
fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
}
