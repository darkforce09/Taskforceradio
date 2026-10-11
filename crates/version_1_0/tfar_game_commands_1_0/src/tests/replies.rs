//! Unit tests for the reply strings, against `processCommand` and `ts_info`
//! (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:72-162`, `:615-629`).

use super::*;

#[test]
fn the_speaking_code_is_talking_then_receiving() {
    // :99-101.
    assert_eq!(speaking_code(false, false), *b"00");
    assert_eq!(speaking_code(false, true), *b"01");
    assert_eq!(speaking_code(true, false), *b"10");
    assert_eq!(speaking_code(true, true), *b"11");
    for talking in [false, true] {
        for receiving in [false, true] {
            assert_eq!(
                speaking_code_text(talking, receiving).as_bytes(),
                speaking_code(talking, receiving)
            );
        }
    }
}

#[test]
fn the_bulk_reply_keeps_the_trailing_tab() {
    // :114-118 and :122-141.
    assert_eq!(
        bulk_speaking_reply([speaking_code(true, false), UNKNOWN_CLIENT_SPEAKING_CODE]),
        "10\t00\t"
    );
    assert_eq!(bulk_speaking_reply([]), "");
}
