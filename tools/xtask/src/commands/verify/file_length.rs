//! The `file-length` gate: production `.rs` files within 500 lines, test files within 1000.
//!
//! **Role:** counts the lines of every Rust source under crates and tools.
//!
//! **Position:** a repository gate; the ceilings come from `layout`.
//!
//! **Signals and state:** none.
//!
//! **Invariants:** a file under a `tests` folder is a test file; there is no allow-list.

#[cfg(test)]
#[path = "tests/file_length.rs"]
mod tests;

use anyhow::Result;

use crate::layout::{PRODUCTION_FILE_MAX_LINES, TEST_FILE_MAX_LINES};
use crate::support::repository::Repository;

use super::gate::Violation;
use super::source_files::{is_test_file, rust_files};

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for file in rust_files(repository)? {
        let line_count = repository.read_text(&file)?.lines().count();
        let (limit, kind) = if is_test_file(&file) {
            (TEST_FILE_MAX_LINES, "test")
        } else {
            (PRODUCTION_FILE_MAX_LINES, "production")
        };
        if line_count > limit {
            violations.push(Violation::new(
                &file,
                limit + 1,
                format!("{line_count} lines; a {kind} file stays within {limit}: split it by responsibility"),
            ));
        }
    }
    Ok(violations)
}
