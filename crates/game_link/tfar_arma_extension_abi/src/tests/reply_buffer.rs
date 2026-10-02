//! Unit tests for `write_reply`.

use super::*;

#[test]
fn an_empty_buffer_is_left_untouched() {
    let mut backing = [0xAA_u8; 4];
    write_reply(&mut backing[..0], "PONG");
    assert_eq!(backing, [0xAA; 4]);
}

#[test]
fn a_one_byte_buffer_holds_only_the_nul() {
    let mut output = [0xAA_u8; 1];
    write_reply(&mut output, "PONG");
    assert_eq!(output, [0]);
}

#[test]
fn a_reply_that_fits_is_written_whole_with_a_nul() {
    let mut output = [0xAA_u8; 8];
    write_reply(&mut output, "PONG");
    assert_eq!(&output[..5], b"PONG\0");
    assert_eq!(
        &output[5..],
        [0xAA; 3],
        "bytes after the NUL stay as they were"
    );
}

#[test]
fn a_reply_of_exactly_length_minus_one_fills_the_buffer() {
    let mut output = [0xAA_u8; 5];
    write_reply(&mut output, "PONG");
    assert_eq!(output, *b"PONG\0");
}

#[test]
fn a_reply_of_exactly_the_buffer_length_loses_its_last_byte() {
    let mut output = [0xAA_u8; 4];
    write_reply(&mut output, "PONG");
    assert_eq!(output, *b"PON\0");
}

#[test]
fn a_long_reply_is_cut_to_length_minus_one() {
    let mut output = [0xAA_u8; 3];
    write_reply(&mut output, "NOT_SPEAKING");
    assert_eq!(output, *b"NO\0");
}

#[test]
fn a_cut_inside_a_two_byte_character_falls_back_to_its_start() {
    // "aé" is `61 C3 A9`; a 3-byte buffer keeps 2 bytes, which would split "é".
    let mut output = [0xAA_u8; 3];
    write_reply(&mut output, "aé");
    assert_eq!(&output[..2], b"a\0");
}

#[test]
fn a_cut_inside_a_four_byte_character_falls_back_to_its_start() {
    // "a😀" is `61 F0 9F 98 80`; buffers of 3, 4 and 5 bytes all keep only "a".
    for size in 3..=5 {
        let mut output = vec![0xAA_u8; size];
        write_reply(&mut output, "a😀");
        assert_eq!(&output[..2], b"a\0", "buffer of {size} bytes");
    }
    let mut output = [0xAA_u8; 6];
    write_reply(&mut output, "a😀");
    assert_eq!(output, *"a😀\0".as_bytes());
}

#[test]
fn an_empty_reply_writes_only_the_nul() {
    let mut output = [0xAA_u8; 4];
    write_reply(&mut output, "");
    assert_eq!(output, [0, 0xAA, 0xAA, 0xAA]);
}
