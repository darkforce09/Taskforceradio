//! Unit tests for the string, list and buffer helpers of `owned_memory`.

use std::ffi::c_char;
use std::path::Path;

use super::Ts3;
use super::owned_memory::{borrowed_text, path_from_buffer};
use crate::fake_ts3_table::{
    fake_table, malloc_bytes, malloc_list, malloc_string, take_freed_pointers,
};

fn fake_ts3() -> Ts3 {
    let _ = take_freed_pointers();
    Ts3::from_functions(fake_table())
}

#[test]
fn a_taken_string_is_freed_through_free_memory() {
    let ts3 = fake_ts3();
    let text = malloc_string("[B]Yes[/B]");
    // SAFETY: `text` is a fresh `malloc`ed string handed over to `take_string`.
    let copied = unsafe { ts3.take_string(text) };
    assert_eq!(copied, "[B]Yes[/B]");
    assert_eq!(take_freed_pointers(), vec![text as usize]);
}

#[test]
fn a_null_string_is_empty_and_frees_nothing() {
    let ts3 = fake_ts3();
    // SAFETY: null is allowed.
    let copied = unsafe { ts3.take_string(std::ptr::null_mut()) };
    assert_eq!(copied, "");
    assert_eq!(take_freed_pointers(), Vec::<usize>::new());
}

#[test]
fn invalid_utf8_from_teamspeak_is_read_lossily() {
    let ts3 = fake_ts3();
    let text = malloc_bytes(b"ab\xffc\0").cast::<c_char>();
    // SAFETY: `text` is a fresh `malloc`ed NUL-terminated string handed over to `take_string`.
    let copied = unsafe { ts3.take_string(text) };
    assert_eq!(copied, "ab\u{fffd}c");
    assert_eq!(take_freed_pointers().len(), 1);
}

#[test]
fn a_list_stops_at_its_first_zero_even_with_data_after_it() {
    let ts3 = fake_ts3();
    let list = malloc_list(&[5_u16, 0, 9, 0]);
    // SAFETY: `list` is a fresh `malloc`ed, 0-terminated array handed over.
    let items = unsafe { ts3.take_terminated_list(list) };
    assert_eq!(items, vec![5]);
    assert_eq!(take_freed_pointers(), vec![list as usize]);
}

#[test]
fn an_empty_list_holds_only_the_terminator() {
    let ts3 = fake_ts3();
    let list = malloc_list(&[0_u64]);
    // SAFETY: `list` is a fresh `malloc`ed, 0-terminated array handed over.
    let items = unsafe { ts3.take_terminated_list(list) };
    assert_eq!(items, Vec::<u64>::new());
    assert_eq!(take_freed_pointers().len(), 1);
}

#[test]
fn a_borrowed_null_string_reads_as_empty() {
    // SAFETY: null is allowed.
    assert_eq!(unsafe { borrowed_text(std::ptr::null()) }, "");
    // SAFETY: a static NUL-terminated string.
    assert_eq!(unsafe { borrowed_text(c"TFAR".as_ptr()) }, "TFAR");
}

#[test]
fn a_path_buffer_ends_at_its_first_nul_or_its_end() {
    assert_eq!(path_from_buffer(b"/a/b\0junk"), Path::new("/a/b"));
    assert_eq!(
        path_from_buffer(b"/no/terminator"),
        Path::new("/no/terminator")
    );
    assert_eq!(path_from_buffer(b"\0"), Path::new(""));
}
