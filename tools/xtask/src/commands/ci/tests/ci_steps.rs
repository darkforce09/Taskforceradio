//! Tests for the `ci-local` step table.

use super::*;

#[test]
fn the_toolchain_steps_come_first_in_the_laws_order_then_every_gate() {
    let names: Vec<String> = ci_local_steps().into_iter().map(|step| step.name).collect();
    assert_eq!(
        names[..6],
        ["fmt", "clippy", "clippy-windows", "test", "deny", "typos"]
    );
    let gates: Vec<&str> = Gate::ALL.iter().map(|gate| gate.name()).collect();
    assert_eq!(names[6..], gates[..]);
}

#[test]
fn step_names_are_unique() {
    let mut names: Vec<String> = ci_local_steps().into_iter().map(|step| step.name).collect();
    let count = names.len();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), count);
}
