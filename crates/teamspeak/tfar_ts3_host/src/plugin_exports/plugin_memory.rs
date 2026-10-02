//! Memory the plugin hands TeamSpeak and TeamSpeak hands back: the `infoData` text, allocated with
//! `malloc` and released by `ts3plugin_freeMemory` with `free`.
//!
//! The C allocator is used, not Rust's, because `ts3plugin_freeMemory` receives only a pointer:
//! `free` needs no size, and TeamSpeak's own plugins pair `malloc` with `free` the same way.

use std::ffi::{c_char, c_void};

/// Copies `text` into a new `malloc` block as a NUL-terminated C string.
///
/// The text is cut at its first NUL byte, since TeamSpeak reads only up to there. Returns null
/// when `malloc` fails; TeamSpeak treats a null `infoData` result as "no text".
pub(crate) fn malloc_c_string(text: &str) -> *mut c_char {
    let bytes = text.as_bytes();
    let length = bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len());
    let Some(size) = length.checked_add(1) else {
        return std::ptr::null_mut();
    };
    // SAFETY: `malloc` has no precondition; the result is checked for null before use.
    let block = unsafe { libc::malloc(size) }.cast::<c_char>();
    if block.is_null() {
        return block;
    }
    // SAFETY: `block` is a fresh allocation of `length + 1` bytes, so `length` bytes of `bytes`
    // (which holds at least `length`) fit, and the NUL goes into the last byte; the regions are
    // distinct because `block` was just allocated.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), block, length);
        block.add(length).write(0);
    }
    block
}

/// Releases a block from [`malloc_c_string`]; null is ignored, as `free` ignores it.
///
/// # Safety
///
/// `block` is null or a pointer this crate returned from [`malloc_c_string`] (or another `malloc`
/// of the same C library) that has not been freed yet; TeamSpeak passes back exactly the pointers
/// the plugin handed it.
pub(crate) unsafe fn free_c_block(block: *mut c_void) {
    // SAFETY: the caller guarantees `block` is null or a live `malloc` allocation, which is what
    // `free` requires; ownership ends here.
    unsafe { libc::free(block) }
}
