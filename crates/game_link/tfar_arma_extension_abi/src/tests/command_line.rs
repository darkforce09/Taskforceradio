//! Unit tests for `arma_command_line_contains`.

use super::*;

#[test]
fn the_command_line_contains_the_program_path() {
    let program = std::env::args().next().unwrap();
    assert!(arma_command_line_contains(&program));
}

#[test]
fn a_flag_that_was_not_passed_is_not_found() {
    assert!(!arma_command_line_contains(
        "-tfdebug-never-passed-to-the-test-runner"
    ));
}
