//! Unit tests for the push-to-talk gate.

use super::*;

#[test]
fn a_press_with_no_release_waiting_switches_the_microphone_on() {
    let mut gate = TangentGate::default();
    assert!(gate.press());
    assert!(!gate.is_release_skipped);
}

#[test]
fn a_release_after_its_delay_switches_the_microphone_back() {
    let mut gate = TangentGate::default();
    assert!(gate.press());
    gate.begin_release();
    assert!(gate.is_waiting_for_release);
    assert!(gate.finish_release());
    assert!(!gate.is_waiting_for_release);
}

#[test]
fn a_press_while_a_release_waits_skips_that_release_and_keeps_the_microphone_on() {
    let mut gate = TangentGate::default();
    assert!(gate.press());
    gate.begin_release();
    assert!(
        !gate.press(),
        "the microphone is still on, so nothing is sent"
    );
    assert!(!gate.finish_release(), "the skipped release does nothing");
    assert!(gate.is_waiting_for_release);
    gate.begin_release();
    assert!(gate.finish_release(), "the next release acts");
    assert!(!gate.is_waiting_for_release);
}
