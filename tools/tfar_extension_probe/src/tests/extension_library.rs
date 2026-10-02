//! Unit tests for reading a reply out of the output buffer, and for a library that is missing.

use super::*;

#[test]
fn the_buffer_is_the_size_arma_passes() {
    assert_eq!(OUTPUT_BUFFER_SIZE, 10240);
    assert_eq!(buffer_size_argument(), 10240);
}

#[test]
fn the_reply_ends_at_the_first_nul() {
    assert_eq!(reply_from_buffer(b"PONG\0garbage\0"), "PONG");
    assert_eq!(reply_from_buffer(b"\0PONG"), "");
}

#[test]
fn a_buffer_without_nul_is_read_whole() {
    assert_eq!(reply_from_buffer(b"OK"), "OK");
}

#[test]
fn a_reply_that_is_not_utf8_is_decoded_lossily() {
    assert_eq!(reply_from_buffer(&[b'O', 0xFF, b'K', 0]), "O\u{fffd}K");
}

#[test]
fn a_missing_library_is_an_error_naming_the_path() {
    let Err(error) =
        ExtensionLibrary::load(Path::new("/nonexistent/task_force_radio_pipe_x64.dll"))
    else {
        panic!("a missing library loaded");
    };
    assert!(format!("{error:#}").contains("/nonexistent/task_force_radio_pipe_x64.dll"));
}
