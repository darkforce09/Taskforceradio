//! Unit tests for the reply strings, against `processCommand` and `ts_info`
//! (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:72-162`, `:615-629`).

use super::*;

#[test]
fn the_reply_strings_are_the_originals() {
    assert_eq!(UNKNOWN_COMMAND, "UNKNOWN COMMAND"); // :80
    assert_eq!(
        ASYNC_COMMAND_SENT_IN_SYNC_CONTEXT,
        "ASYNC COMMAND SENT IN SYNC CONTEXT"
    ); // :161
    assert_eq!(FAIL, "FAIL"); // :628
    assert_eq!(PONG, "PONG"); // :625
    assert_eq!(ERROR_GETTING_SERVER_NAME, "ERROR_GETTING_SERVER_NAME"); // Teamspeak.cpp:555
    assert_eq!(CHANNEL_NAME_UNAVAILABLE, ""); // Teamspeak.cpp:534
}

#[test]
fn the_speaking_code_is_talking_then_receiving() {
    // :99-101.
    assert_eq!(speaking_code(false, false), *b"00");
    assert_eq!(speaking_code(false, true), *b"01");
    assert_eq!(speaking_code(true, false), *b"10");
    assert_eq!(speaking_code(true, true), *b"11");
    assert_eq!(UNKNOWN_CLIENT_SPEAKING_CODE, *b"00"); // :92-95
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
