//! Unit tests for the reception vocabulary.

use super::*;

#[test]
fn only_short_and_long_range_are_carried_radios() {
    assert!(ReceivedOn::ShortRange.is_carried_radio());
    assert!(ReceivedOn::LongRange.is_carried_radio());
    assert!(!ReceivedOn::Ground.is_carried_radio());
    assert!(!ReceivedOn::Intercom.is_carried_radio());
}

#[test]
fn ignore_tangents_default_to_off() {
    assert_eq!(IgnoreTangents::default(), IgnoreTangents::NONE);
}
