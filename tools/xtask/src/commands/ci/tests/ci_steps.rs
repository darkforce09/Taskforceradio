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
fn the_windows_clippy_step_lints_the_extension_side() {
    let steps = ci_local_steps();
    let windows = steps
        .iter()
        .find(|step| step.name == "clippy-windows")
        .unwrap();
    assert_eq!(
        windows.command_line(),
        "cargo clippy -p tfar_extension_0_9 -p tfar_extension_1_0 -p tfar_arma_extension_abi \
         -p tfar_game_link_client -p tfar_game_link_protocol -p tfar_extension_probe \
         --target x86_64-pc-windows-gnu -- -D warnings"
    );
}

#[test]
fn step_names_are_unique() {
    let mut names: Vec<String> = ci_local_steps().into_iter().map(|step| step.name).collect();
    let count = names.len();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), count);
}
