//! Unit tests for the stub game link handler.

use super::*;

#[test]
fn the_ping_is_answered_with_pong() {
    assert_eq!(StubGameLinkHandler.sync_request(b"TS_INFO\tPING"), b"PONG");
}

#[test]
fn every_other_sync_call_gets_unknown_command() {
    for text in [
        &b"TS_INFO\tVERSION"[..],
        b"TS_INFO\tPING\t",
        b"POS\tplayer",
        b"",
    ] {
        assert_eq!(StubGameLinkHandler.sync_request(text), b"UNKNOWN COMMAND");
    }
}
