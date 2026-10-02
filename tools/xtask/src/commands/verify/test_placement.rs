//! The `test-placement` gate: unit tests only in sibling files under `tests/`.
//!
//! **Role:** finds inline test modules, `#[test]` functions and test modules attached without a
//! `#[path = "tests/…"]` attribute in production Rust files.
//!
//! **Position:** a repository gate; reads sources through `rust_source`.
//!
//! **Signals and state:** none.
//!
//! **Invariants:** a production file is any `.rs` file not under a `tests` folder; inside a test
//! file anything goes.

#[cfg(test)]
#[path = "tests/test_placement.rs"]
mod tests;

use anyhow::Result;

use crate::support::repository::Repository;
use crate::support::rust_source;

use super::gate::Violation;
use super::source_files::{is_test_file, rust_files};

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for file in rust_files(repository)? {
        if is_test_file(&file) {
            continue;
        }
        let text = repository.read_text(&file)?;
        let raw_lines: Vec<&str> = text.lines().collect();
        let code_lines = rust_source::code_lines(&text);
        for (index, code) in code_lines.iter().enumerate() {
            let trimmed = code.trim();
            if trimmed.starts_with("#[test]") {
                violations.push(Violation::new(
                    &file,
                    index + 1,
                    "a `#[test]` function in a production file; move it to a sibling file under `tests/`",
                ));
                continue;
            }
            let Some((name, is_inline)) = module_declaration(trimmed) else {
                continue;
            };
            let attributes = attribute_lines_above(&raw_lines, index);
            let is_test_module = name == "tests"
                || attributes
                    .iter()
                    .any(|attribute| attribute.contains("cfg(test)"));
            if !is_test_module {
                continue;
            }
            if is_inline {
                violations.push(Violation::new(
                    &file,
                    index + 1,
                    format!("inline test module `mod {name} {{`; attach a sibling file with `#[path = \"tests/<file>.rs\"]`"),
                ));
            } else if !attributes
                .iter()
                .any(|attribute| attribute.contains("#[path = \"tests/"))
            {
                violations.push(Violation::new(
                    &file,
                    index + 1,
                    format!(
                        "test module `{name}` must attach with `#[path = \"tests/<file>.rs\"]`"
                    ),
                ));
            }
        }
    }
    Ok(violations)
}

/// When the code line declares a module: its name and whether its body is inline.
fn module_declaration(code: &str) -> Option<(&str, bool)> {
    let without_visibility = ["pub(crate) ", "pub(super) ", "pub "]
        .iter()
        .find_map(|prefix| code.strip_prefix(prefix))
        .unwrap_or(code);
    let rest = without_visibility.strip_prefix("mod ")?;
    let name_end = rest
        .find(|character: char| !(character.is_alphanumeric() || character == '_'))
        .unwrap_or(rest.len());
    let name = rest.get(..name_end)?;
    let after = rest.get(name_end..)?.trim_start();
    if after.starts_with(';') {
        Some((name, false))
    } else if after.starts_with('{') || after.is_empty() {
        Some((name, true))
    } else {
        None
    }
}

/// The raw attribute lines directly above line `index`, skipping comment lines.
fn attribute_lines_above<'text>(raw_lines: &[&'text str], index: usize) -> Vec<&'text str> {
    let mut found = Vec::new();
    let mut cursor = index;
    while let Some(above) = cursor.checked_sub(1) {
        let Some(line) = raw_lines.get(above).map(|line| line.trim()) else {
            break;
        };
        if line.starts_with("#[") {
            found.push(line);
        } else if !rust_source::is_comment_line(line) {
            break;
        }
        cursor = above;
    }
    found
}
