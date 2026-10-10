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
    let generation = gate.begin_release();
    assert!(gate.is_waiting_for_release);
    assert!(gate.finish_release(generation));
    assert!(!gate.is_waiting_for_release);
}

#[test]
fn a_press_while_a_release_waits_skips_that_release_and_keeps_the_microphone_on() {
    let mut gate = TangentGate::default();
    assert!(gate.press());
    let first = gate.begin_release();
    assert!(
        !gate.press(),
        "the microphone is still on, so nothing is sent"
    );
    assert!(
        !gate.finish_release(first),
        "the skipped release does nothing"
    );
    assert!(gate.is_waiting_for_release);
    let second = gate.begin_release();
    assert!(gate.finish_release(second), "the next release acts");
    assert!(!gate.is_waiting_for_release);
}

#[test]
fn a_game_end_cancels_every_waiting_release_and_the_next_press_starts_afresh() {
    let mut gate = TangentGate::default();
    assert!(gate.press());
    let first = gate.begin_release();
    assert!(!gate.press());
    let second = gate.begin_release();
    gate.cancel_releases();
    assert!(!gate.is_waiting_for_release);
    assert!(!gate.is_release_skipped);
    assert!(
        !gate.finish_release(first),
        "a release from before the game end"
    );
    assert!(
        !gate.finish_release(second),
        "a release from before the game end"
    );
    assert!(
        gate.press(),
        "the next press switches the microphone on again"
    );
    let after = gate.begin_release();
    assert!(
        gate.finish_release(after),
        "a release after the game end acts"
    );
}
