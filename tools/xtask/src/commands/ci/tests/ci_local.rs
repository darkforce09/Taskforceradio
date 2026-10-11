//! Tests for the `ci-local` runner: outcomes, logs and the stop at the first failure.

use super::*;
use crate::commands::ci::ci_steps::StepProgram;
use crate::fixture_tree::FixtureTree;

fn tool(name: &str, program: &str, arguments: &[&str]) -> CiStep {
    CiStep {
        name: name.to_owned(),
        program: StepProgram::Tool(
            program.to_owned(),
            arguments
                .iter()
                .map(|argument| (*argument).to_owned())
                .collect(),
        ),
    }
}

#[test]
fn the_lane_stops_at_the_first_failure_and_logs_each_step() {
    let fixture = FixtureTree::empty();
    let logs = fixture.root().join(".workstation/logs/ci");
    let steps = [
        tool("version", "git", &["--version"]),
        tool("broken", "git", &["no-such-git-command"]),
        tool("after", "git", &["--version"]),
    ];
    let mut out = Vec::new();
    let outcomes = run_steps(&steps, fixture.root(), Path::new("xtask"), &logs, &mut out).unwrap();
    assert_eq!(
        outcomes,
        [StepOutcome::Held, StepOutcome::Failed, StepOutcome::NotRun]
    );
    assert_eq!(exit_code(&outcomes), 1);
    let printed = String::from_utf8(out).unwrap();
    assert!(
        printed.contains("version") && printed.contains("held"),
        "{printed}"
    );
    assert!(
        printed.contains("after") && printed.contains("not run"),
        "{printed}"
    );
    let first_log = std::fs::read_to_string(logs.join("01-version.log")).unwrap();
    assert!(
        first_log.starts_with("$ git --version") && first_log.contains("git version"),
        "{first_log}"
    );
    assert!(!logs.join("03-after.log").exists());
}

#[test]
fn a_missing_tool_is_not_run_never_held() {
    let fixture = FixtureTree::empty();
    let logs = fixture.root().join(".workstation/logs/ci");
    let steps = [tool("missing", "xtask-no-such-tool", &[])];
    let outcomes = run_steps(
        &steps,
        fixture.root(),
        Path::new("xtask"),
        &logs,
        &mut Vec::new(),
    )
    .unwrap();
    assert_eq!(outcomes, [StepOutcome::NotRun]);
    assert_eq!(exit_code(&outcomes), 2);
}

#[test]
fn every_step_held_exits_0() {
    assert_eq!(exit_code(&[StepOutcome::Held, StepOutcome::Held]), 0);
}
