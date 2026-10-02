//! Unit tests for the `malloc`ed `infoData` text and its release.

use std::ffi::CStr;

use super::plugin_memory::{free_c_block, malloc_c_string};

#[test]
fn info_text_is_copied_into_a_nul_terminated_malloc_block() {
    let block = malloc_c_string("Connected to Game: [B]Yes[/B]");
    assert!(!block.is_null());
    // SAFETY: `block` is the NUL-terminated string just allocated.
    let text = unsafe { CStr::from_ptr(block) };
    assert_eq!(text.to_str().unwrap(), "Connected to Game: [B]Yes[/B]");
    // SAFETY: `block` came from `malloc_c_string` and is freed once.
    unsafe { free_c_block(block.cast()) };
}

#[test]
fn info_text_stops_at_an_interior_nul() {
    let block = malloc_c_string("before\0after");
    // SAFETY: `block` is the NUL-terminated string just allocated.
    let text = unsafe { CStr::from_ptr(block) };
    assert_eq!(text.to_bytes(), b"before");
    // SAFETY: `block` came from `malloc_c_string` and is freed once.
    unsafe { free_c_block(block.cast()) };
}

#[test]
fn empty_info_text_is_a_lone_nul() {
    let block = malloc_c_string("");
    // SAFETY: `block` is the NUL-terminated string just allocated.
    assert_eq!(unsafe { CStr::from_ptr(block) }.to_bytes(), b"");
    // SAFETY: `block` came from `malloc_c_string` and is freed once.
    unsafe { free_c_block(block.cast()) };
}

#[test]
fn freeing_null_is_a_no_op() {
    // SAFETY: null is allowed, as for `free`.
    unsafe { free_c_block(std::ptr::null_mut()) };
}
