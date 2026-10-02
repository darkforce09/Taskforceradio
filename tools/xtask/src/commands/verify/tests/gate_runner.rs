//! Tests for the gate runner: output lines and verdicts.

use super::*;
use crate::fixture_tree::FixtureTree;

#[test]
fn a_held_gate_prints_no_violation_and_exits_0() {
    let fixture = FixtureTree::sample_workspace();
    let (mut violations, mut verdicts) = (Vec::new(), Vec::new());
    let verdict = run_gate(
        Gate::LanguageBans,
        &fixture.repository(),
        &mut violations,
        &mut verdicts,
    );
    assert_eq!(verdict, Verdict::Held);
    assert_eq!(verdict.exit_code(), 0);
    assert_eq!(violations, Vec::<u8>::new());
    assert_eq!(
        String::from_utf8(verdicts).unwrap(),
        "language-bans: held\n"
    );
}

#[test]
fn a_violation_prints_path_line_gate_message_and_exits_1() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write("tools/helper.py", "x\n");
    let (mut violations, mut verdicts) = (Vec::new(), Vec::new());
    let verdict = run_gate(
        Gate::LanguageBans,
        &fixture.repository(),
        &mut violations,
        &mut verdicts,
    );
    assert_eq!(verdict.exit_code(), 1);
    let printed = String::from_utf8(violations).unwrap();
    assert!(
        printed.starts_with("tools/helper.py:1: language-bans: no Python"),
        "{printed}"
    );
}

#[test]
fn a_gate_that_could_not_run_exits_2_and_never_says_held() {
    let fixture = FixtureTree::sample_workspace();
    fixture.remove("reference/MANIFEST.sha256");
    let (mut violations, mut verdicts) = (Vec::new(), Vec::new());
    let verdict = run_gate(
        Gate::ReferenceUntouched,
        &fixture.repository(),
        &mut violations,
        &mut verdicts,
    );
    assert_eq!(verdict.exit_code(), 2);
    let printed = String::from_utf8(verdicts).unwrap();
    assert!(
        printed.contains("could not run") && !printed.contains("held"),
        "{printed}"
    );
}

#[test]
fn the_worst_verdict_wins() {
    assert!(Verdict::CouldNotRun > Verdict::Violated && Verdict::Violated > Verdict::Held);
}

#[test]
fn every_gate_holds_on_the_sample_workspace() {
    let fixture = FixtureTree::sample_workspace();
    for gate in Gate::ALL {
        let (mut violations, mut verdicts) = (Vec::new(), Vec::new());
        let verdict = run_gate(gate, &fixture.repository(), &mut violations, &mut verdicts);
        assert_eq!(
            verdict,
            Verdict::Held,
            "{}: {}",
            gate.name(),
            String::from_utf8_lossy(&violations)
        );
    }
}
