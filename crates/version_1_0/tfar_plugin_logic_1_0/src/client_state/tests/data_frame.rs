//! Unit tests for `DataFrame`.

use super::*;

#[test]
fn the_invalid_frame_stays_invalid_when_advanced() {
    assert_eq!(DataFrame::INVALID.next(), DataFrame::INVALID);
    assert!(!DataFrame::INVALID.is_valid());
    assert_eq!(DataFrame::default(), DataFrame::INVALID);
}

#[test]
fn frames_advance_by_one_and_saturate() {
    assert_eq!(DataFrame::FIRST.next(), DataFrame::new(1));
    assert_eq!(DataFrame::new(u32::MAX).next(), DataFrame::new(u32::MAX));
    assert_eq!(DataFrame::new(4).number(), Some(4));
}

#[test]
fn liveness_needs_two_valid_frames_at_most_one_apart() {
    assert!(DataFrame::new(5).is_within_one_of(DataFrame::new(6)));
    assert!(DataFrame::new(6).is_within_one_of(DataFrame::new(5)));
    assert!(DataFrame::new(5).is_within_one_of(DataFrame::new(5)));
    assert!(!DataFrame::new(5).is_within_one_of(DataFrame::new(7)));
    assert!(!DataFrame::INVALID.is_within_one_of(DataFrame::new(0)));
    assert!(!DataFrame::new(0).is_within_one_of(DataFrame::INVALID));
}
