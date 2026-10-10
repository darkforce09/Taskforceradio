//! Unit tests for the audio callback counters.

use super::*;

#[test]
fn new_counters_take_as_zero() {
    let diagnostics = AudioDiagnostics::new();
    assert!(diagnostics.take().is_zero());
}

#[test]
fn a_take_returns_the_counts_and_resets_them() {
    let diagnostics = AudioDiagnostics::new();
    diagnostics.record_contended_state_lock();
    diagnostics.record_contended_state_lock();
    diagnostics.record_skipped_block();
    assert_eq!(
        diagnostics.take(),
        AudioDiagnosticCounts {
            contended_state_locks: 2,
            skipped_blocks: 1,
        }
    );
    assert!(diagnostics.take().is_zero());
}
