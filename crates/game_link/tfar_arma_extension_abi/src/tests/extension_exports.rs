//! Unit tests for the `arma_extension!` exports, called through raw function pointers as Arma calls
//! them.

#![allow(
    unreachable_pub,
    reason = "the expanded exports are `pub` as in an app crate root, here inside a private test module"
)]

use core::ffi::{c_char, c_int};

use super::*;

/// The handler the test expansion uses: echoes the call text behind `reply:`.
fn echo_handler(text: &str) -> String {
    format!("reply:{text}")
}

crate::arma_extension!(handler: echo_handler, version: "test-version-1");

type ExtensionExport = unsafe extern "system" fn(*mut c_char, c_int, *const c_char);
type VersionExport = unsafe extern "system" fn(*mut c_char, c_int);

/// Calls `RVExtension` through a raw function pointer with a Rust-allocated buffer of `size` bytes
/// and returns the buffer.
fn call_extension(input: &[u8], size: usize) -> Vec<u8> {
    let export: ExtensionExport = RVExtension;
    let mut input = input.to_vec();
    input.push(0);
    let mut output = vec![0xAA_u8; size];
    // SAFETY: `output` holds `size` writable bytes and `input` is NUL-terminated; both outlive the
    // call.
    unsafe {
        export(
            output.as_mut_ptr().cast::<c_char>(),
            c_int::try_from(size).unwrap(),
            input.as_ptr().cast::<c_char>(),
        );
    }
    output
}

/// The bytes before the first NUL of `output`.
fn reply_text(output: &[u8]) -> &[u8] {
    let end = output.iter().position(|byte| *byte == 0).unwrap();
    &output[..end]
}

#[test]
fn the_export_passes_the_call_text_to_the_handler_and_writes_its_reply() {
    let output = call_extension(b"TS_INFO\tPING", 64);
    assert_eq!(reply_text(&output), b"reply:TS_INFO\tPING");
}

#[test]
fn the_export_truncates_the_reply_to_the_output_size() {
    let output = call_extension(b"PING", 6);
    assert_eq!(output, *b"reply\0");
}

#[test]
fn the_export_decodes_invalid_utf8_lossily_and_keeps_valid_bytes() {
    let output = call_extension(b"\xC3\xA9\xFF", 64);
    assert_eq!(reply_text(&output), "reply:é\u{FFFD}".as_bytes());
}

#[test]
fn the_export_writes_nothing_for_a_null_output() {
    let export: ExtensionExport = RVExtension;
    let input = b"PING\0";
    // SAFETY: a null output is part of the contract; `input` is NUL-terminated.
    unsafe { export(core::ptr::null_mut(), 64, input.as_ptr().cast::<c_char>()) };
}

#[test]
fn the_export_writes_nothing_for_a_non_positive_size() {
    let export: ExtensionExport = RVExtension;
    let input = b"PING\0";
    for size in [0, -1, c_int::MIN] {
        let mut output = [0xAA_u8; 8];
        // SAFETY: `output` is valid; the export must not touch it for a size of zero or less.
        unsafe {
            export(
                output.as_mut_ptr().cast::<c_char>(),
                size,
                input.as_ptr().cast::<c_char>(),
            );
        }
        assert_eq!(output, [0xAA; 8], "size {size}");
    }
}

#[test]
fn the_export_writes_an_empty_reply_for_a_null_input() {
    let export: ExtensionExport = RVExtension;
    let mut output = [0xAA_u8; 8];
    // SAFETY: `output` holds 8 writable bytes; a null input is part of the contract.
    unsafe { export(output.as_mut_ptr().cast::<c_char>(), 8, core::ptr::null()) };
    assert_eq!(output[0], 0);
}

#[test]
fn the_version_export_writes_the_version_truncated() {
    let export: VersionExport = RVExtensionVersion;
    let mut output = vec![0xAA_u8; 64];
    // SAFETY: `output` holds 64 writable bytes that outlive the call.
    unsafe { export(output.as_mut_ptr().cast::<c_char>(), 64) };
    assert_eq!(reply_text(&output), b"test-version-1");

    let mut short = [0xAA_u8; 5];
    // SAFETY: `short` holds 5 writable bytes that outlive the call.
    unsafe { export(short.as_mut_ptr().cast::<c_char>(), 5) };
    assert_eq!(short, *b"test\0");
}

#[test]
fn answer_extension_version_ignores_a_null_output() {
    // SAFETY: a null output is part of the contract.
    unsafe { answer_extension_version(core::ptr::null_mut(), 16, "v") };
}
