//! Unit tests for the reply strings and `reply_for_unhandled`, against `processGameCommand` and
//! `ts_info` in `reference/tfar_0_9_12/ts/src/plugin.cpp`.

use super::*;

fn reply_to(message: &str) -> &'static str {
    reply_for_unhandled(&GameCommand::parse(message).unwrap())
}

#[test]
fn the_reply_strings_are_the_originals_byte_for_byte() {
    assert_eq!(OK, "OK"); // plugin.cpp:1783
    assert_eq!(FAIL, "FAIL"); // plugin.cpp:1935
    assert_eq!(DONE, "DONE"); // plugin.cpp:1778
    assert_eq!(PONG, "PONG"); // plugin.cpp:1557
    assert_eq!(SPEAKING, "SPEAKING"); // plugin.cpp:1930
    assert_eq!(NOT_SPEAKING, "NOT_SPEAKING"); // plugin.cpp:1933
    assert_eq!(ERROR_GETTING_SERVER_NAME, "ERROR_GETTING_SERVER_NAME"); // plugin.cpp:1542
    assert_eq!(NO_CHANNEL_NAME, ""); // getChannelName, plugin.cpp:862-873
}

#[test]
fn ts_info_replies_follow_ts_info() {
    assert_eq!(reply_to("TS_INFO\tPING"), "PONG");
    assert_eq!(reply_to("TS_INFO\tSERVER"), "ERROR_GETTING_SERVER_NAME");
    assert_eq!(reply_to("TS_INFO\tCHANNEL"), "");
    assert_eq!(reply_to("TS_INFO\tWHAT"), "FAIL"); // plugin.cpp:1559
}

#[test]
fn killed_with_the_two_fields_arma_sends_replies_done() {
    // F-020: plugin.cpp:1775 needs more than 2 fields; fn_sendPlayerKilled.sqf sends 2.
    assert_eq!(reply_to("KILLED\tAlpha 1-1"), "DONE");
}

#[test]
fn release_all_tangents_and_track_reply_ok() {
    // F-021: plugin.cpp:1785 and 1799 have no return and fall through to FAIL at 1935.
    assert_eq!(reply_to("RELEASE_ALL_TANGENTS\tAlpha"), "OK");
    assert_eq!(reply_to("TRACK\tpiwik.php?idsite=2"), "OK");
}

#[test]
fn the_fixed_ok_replies_match_the_original() {
    assert_eq!(reply_to("SPEAKERS"), "OK"); // plugin.cpp:1783
    assert_eq!(reply_to("VERSION\t0.9.12\tTaskForceRadio\t123"), "OK"); // plugin.cpp:1797
    assert_eq!(reply_to("TANGENT\tPRESSED\t87.6\t5000\tdigital"), "OK"); // plugin.cpp:1872
    assert_eq!(
        reply_to("FREQ\t[]\t[]\tNo_DD_Radio\ttrue\t20\t7\tAlpha\t0\t7\t1\t1\t1\t20"),
        "OK"
    ); // plugin.cpp:1907
}

#[test]
fn position_and_is_speaking_fall_back_to_not_speaking() {
    assert_eq!(reply_to("IS_SPEAKING\tAlpha"), "NOT_SPEAKING"); // plugin.cpp:1933
    assert_eq!(
        reply_to("POS\tAlpha\t1\t2\t3\t90\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t180"),
        "NOT_SPEAKING"
    ); // plugin.cpp:1677
}

#[test]
fn an_unknown_message_and_a_parse_error_reply_fail() {
    assert_eq!(reply_to("HELLO\tWORLD"), "FAIL");
    assert_eq!(GameCommand::parse("TRACK").unwrap_err().reply(), "FAIL");
}
