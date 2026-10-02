//! Unit tests for the parser of the `Capture/Default/PreProcessing` profile text.

use std::time::Duration;

use super::*;

#[test]
fn an_enabled_delay_with_milliseconds_is_read() {
    let delay =
        PushToTalkDelay::parse_pre_processing("agc=true\ndelay_ptt=true\ndelay_ptt_msecs=250\n")
            .unwrap();
    assert_eq!(
        delay,
        PushToTalkDelay {
            enabled: true,
            milliseconds: 250
        }
    );
    assert_eq!(delay.effective(), Duration::from_millis(250));
}

#[test]
fn a_disabled_delay_keeps_its_milliseconds_but_is_zero_in_effect() {
    let delay =
        PushToTalkDelay::parse_pre_processing("delay_ptt=false\ndelay_ptt_msecs=500").unwrap();
    assert_eq!(
        delay,
        PushToTalkDelay {
            enabled: false,
            milliseconds: 500
        }
    );
    assert_eq!(delay.effective(), Duration::ZERO);
}

#[test]
fn crlf_line_ends_are_read_like_lf() {
    let delay =
        PushToTalkDelay::parse_pre_processing("delay_ptt=true\r\ndelay_ptt_msecs=300\r\n").unwrap();
    assert_eq!(delay.effective(), Duration::from_millis(300));
}

#[test]
fn a_profile_without_the_two_lines_is_a_disabled_zero_delay() {
    let delay = PushToTalkDelay::parse_pre_processing("garbage\n\u{1}\u{2}==\nvad=true").unwrap();
    assert_eq!(delay, PushToTalkDelay::default());
    assert_eq!(delay.effective(), Duration::ZERO);
}

#[test]
fn garbage_milliseconds_are_an_error() {
    let error = PushToTalkDelay::parse_pre_processing("delay_ptt=true\ndelay_ptt_msecs=soon\r\n")
        .unwrap_err();
    assert!(
        matches!(&error, Error::InvalidMilliseconds { line } if line == "delay_ptt_msecs=soon"),
        "{error:?}"
    );
}

#[test]
fn negative_milliseconds_are_an_error() {
    let error = PushToTalkDelay::parse_pre_processing("delay_ptt_msecs=-5").unwrap_err();
    assert!(matches!(error, Error::InvalidMilliseconds { .. }));
}

#[test]
fn a_milliseconds_key_without_a_value_is_an_error() {
    let error = PushToTalkDelay::parse_pre_processing("delay_ptt_msecs").unwrap_err();
    assert!(matches!(error, Error::InvalidMilliseconds { .. }));
}

#[test]
fn the_last_line_of_each_kind_wins() {
    let delay =
        PushToTalkDelay::parse_pre_processing("delay_ptt_msecs=100\ndelay_ptt_msecs=700").unwrap();
    assert_eq!(delay.milliseconds, 700);
}
