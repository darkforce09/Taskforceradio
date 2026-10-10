//! Unit tests for `FocusWatch`: the read interval and the change test.

use std::cell::Cell;

use super::*;

#[test]
fn the_first_poll_reads_and_reports_the_focus() {
    let mut watch = FocusWatch::new();
    assert_eq!(watch.poll(Instant::now(), || false), Some(false));
}

#[test]
fn a_poll_inside_the_interval_does_not_read() {
    let mut watch = FocusWatch::new();
    let start = Instant::now();
    assert_eq!(watch.poll(start, || true), Some(true));
    let reads = Cell::new(0);
    let read = || {
        reads.set(reads.get() + 1);
        false
    };
    assert_eq!(watch.poll(start + Duration::from_millis(249), read), None);
    assert_eq!(reads.get(), 0);
}

#[test]
fn an_unchanged_focus_is_not_reported_again() {
    let mut watch = FocusWatch::new();
    let start = Instant::now();
    assert_eq!(watch.poll(start, || true), Some(true));
    assert_eq!(watch.poll(start + FOCUS_READ_INTERVAL, || true), None);
}

#[test]
fn a_changed_focus_is_reported_once_the_interval_passed() {
    let mut watch = FocusWatch::new();
    let start = Instant::now();
    assert_eq!(watch.poll(start, || true), Some(true));
    assert_eq!(
        watch.poll(start + FOCUS_READ_INTERVAL, || false),
        Some(false)
    );
    assert_eq!(watch.poll(start + FOCUS_READ_INTERVAL * 2, || false), None);
    assert_eq!(
        watch.poll(start + FOCUS_READ_INTERVAL * 3, || true),
        Some(true)
    );
}

#[test]
fn the_interval_counts_from_the_last_read_not_the_last_report() {
    let mut watch = FocusWatch::new();
    let start = Instant::now();
    assert_eq!(watch.poll(start, || true), Some(true));
    let second_read = start + Duration::from_millis(300);
    assert_eq!(watch.poll(second_read, || true), None);
    assert_eq!(
        watch.poll(second_read + Duration::from_millis(200), || false),
        None
    );
    assert_eq!(
        watch.poll(second_read + FOCUS_READ_INTERVAL, || false),
        Some(false)
    );
}
