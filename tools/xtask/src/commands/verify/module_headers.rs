//! The `module-headers` gate: the four-point `//!` header on crate roots and long files.
//!
//! **Role:** checks that every crate root and every production file of 80 lines or more opens
//! with a `//!` summary line followed by the Role, Position, Signals and state, and Invariants
//! paragraphs, in that order.
//!
//! **Position:** a repository gate; the labels and the line threshold come from `layout`.
//!
//! **Signals and state:** none.
//!
//! **Invariants:** each label opens its own `//!` paragraph (the `//!` line above it is empty);
//! the header is the file's first lines.

#[cfg(test)]
#[path = "tests/module_headers.rs"]
mod tests;

use anyhow::Result;

use crate::layout::{HEADER_REQUIRED_FROM_LINES, MODULE_HEADER_LABELS};
use crate::support::repository::Repository;

use super::gate::Violation;
use super::source_files::{is_crate_root, is_test_file, rust_files};

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for file in rust_files(repository)? {
        if is_test_file(&file) {
            continue;
        }
        let text = repository.read_text(&file)?;
        let is_root = is_crate_root(repository, &file)?;
        if !is_root && text.lines().count() < HEADER_REQUIRED_FROM_LINES {
            continue;
        }
        if let Some((line, problem)) = header_problem(&text) {
            let why = if is_root {
                "a crate root"
            } else {
                "a file of 80 lines or more"
            };
            violations.push(Violation::new(
                &file,
                line,
                format!("{why} needs the four-point `//!` header: {problem}"),
            ));
        }
    }
    Ok(violations)
}

/// What is missing from a file's header, with the line to look at, or `None` when it is whole.
pub(crate) fn header_problem(text: &str) -> Option<(usize, String)> {
    let header: Vec<&str> = text
        .lines()
        .take_while(|line| line.starts_with("//!"))
        .collect();
    let summary = header
        .first()
        .map(|line| line.trim_start_matches("//!").trim());
    if summary.is_none_or(|summary| summary.is_empty() || summary.starts_with("**")) {
        return Some((
            1,
            "the first line is a `//!` summary of the module".to_owned(),
        ));
    }
    let mut search_from = 1;
    for label in MODULE_HEADER_LABELS {
        let found = header
            .iter()
            .enumerate()
            .skip(search_from)
            .find(|(_, line)| {
                line.trim_start_matches("//!")
                    .trim_start()
                    .starts_with(label)
            });
        let Some((index, _)) = found else {
            return Some((
                header.len().max(1),
                format!("no `{label}` paragraph after the ones before it"),
            ));
        };
        let opens_paragraph = index
            .checked_sub(1)
            .and_then(|above| header.get(above))
            .is_some_and(|line| line.trim() == "//!");
        if !opens_paragraph {
            return Some((
                index + 1,
                format!("`{label}` must open its own paragraph after an empty `//!` line"),
            ));
        }
        search_from = index + 1;
    }
    None
}
