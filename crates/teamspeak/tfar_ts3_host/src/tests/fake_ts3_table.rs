//! A fake `TS3Functions` table for the unit tests: every slot aborts unless a test fills it, and
//! the memory functions use the C allocator so the code under test can free what a fake hands out.

use std::cell::RefCell;
use std::ffi::{CString, c_char, c_uint, c_void};
use std::mem::MaybeUninit;

use ts3plugin_sys::ts3functions::Ts3Functions;

/// The stand-in for every function a test did not fill: calling it is a test bug, so it aborts.
extern "C" fn unexpected_teamspeak_call() {
    std::process::abort();
}

thread_local! {
    /// The pointers `fake_free_memory` received on this thread, in order.
    static FREED_POINTERS: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

/// `freeMemory`: records the pointer and frees it with `free`, as the fakes allocate with `malloc`.
pub(crate) extern "C" fn fake_free_memory(pointer: *mut c_void) -> c_uint {
    FREED_POINTERS.with(|freed| freed.borrow_mut().push(pointer as usize));
    // SAFETY: every pointer a fake hands out comes from `malloc` and is freed once, here.
    unsafe { libc::free(pointer) };
    0
}

/// The text `fake_get_error_message` answers with, for code `code`.
pub(crate) fn fake_error_text(code: u32) -> String {
    format!("fake error {code:#06x}")
}

/// `getErrorMessage`: a `malloc`ed `fake error 0x….` text.
pub(crate) extern "C" fn fake_get_error_message(code: c_uint, error: *mut *mut c_char) -> c_uint {
    let text = malloc_string(&fake_error_text(code));
    // SAFETY: the code under test passes a writable `char*` slot.
    unsafe { error.write(text) };
    0
}

/// The table with every slot aborting except `freeMemory` and `getErrorMessage`.
pub(crate) fn fake_table() -> Ts3Functions {
    let mut table = MaybeUninit::<Ts3Functions>::uninit();
    let slot_count = size_of::<Ts3Functions>() / size_of::<extern "C" fn()>();
    let slots = table.as_mut_ptr().cast::<extern "C" fn()>();
    for index in 0..slot_count {
        // SAFETY: `Ts3Functions` is `#[repr(C)]` and made only of function pointers (the layout
        // test checks its size is a whole number of pointers), so slot `index` is in bounds and
        // pointer-aligned.
        unsafe { slots.add(index).write(unexpected_teamspeak_call) };
    }
    // SAFETY: every slot now holds a valid, non-null function pointer. The stub's signature differs
    // from the slot's; calling it is undefined only in theory and aborts at once, and only a test
    // bug calls a slot it did not fill.
    let mut table = unsafe { table.assume_init() };
    table.free_memory = fake_free_memory;
    table.get_error_message = fake_get_error_message;
    table
}

/// Pointers this thread passed to `freeMemory` since the last call, and forgets them.
pub(crate) fn take_freed_pointers() -> Vec<usize> {
    FREED_POINTERS.with(|freed| std::mem::take(&mut *freed.borrow_mut()))
}

/// A `malloc`ed NUL-terminated copy of `text`, as TeamSpeak hands strings out.
pub(crate) fn malloc_string(text: &str) -> *mut c_char {
    malloc_bytes(CString::new(text).unwrap().as_bytes_with_nul()).cast()
}

/// A `malloc`ed copy of `bytes`.
pub(crate) fn malloc_bytes(bytes: &[u8]) -> *mut u8 {
    // SAFETY: `malloc` has no precondition; the result is checked below.
    let block = unsafe { libc::malloc(bytes.len().max(1)) }.cast::<u8>();
    assert!(!block.is_null());
    // SAFETY: `block` holds at least `bytes.len()` fresh bytes, distinct from `bytes`.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), block, bytes.len()) };
    block
}

/// A `malloc`ed copy of `items`, as TeamSpeak hands out its zero-terminated lists.
pub(crate) fn malloc_list<T: Copy>(items: &[T]) -> *mut T {
    let size = size_of_val(items);
    // SAFETY: `malloc` has no precondition; the result is checked below.
    let block = unsafe { libc::malloc(size.max(1)) }.cast::<T>();
    assert!(!block.is_null());
    // SAFETY: `block` holds `items.len()` fresh elements (malloc aligns for any type), distinct
    // from `items`.
    unsafe { std::ptr::copy_nonoverlapping(items.as_ptr(), block, items.len()) };
    block
}
