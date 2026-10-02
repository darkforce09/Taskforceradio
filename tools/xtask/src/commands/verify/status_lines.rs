//! The `status-lines` gate: every document under `documentation/` opens with its status line,
//! and no code README carries one.
//!
//! **Role:** reads the first line of every Markdown file outside `reference/`.
//!
//! **Position:** a repository gate; the three status lines come from `layout`.
//!
//! **Signals and state:** none.
//!
//! **Invariants:** a document's first line is exactly `**Status:** live`, `**Status:** archived`
//! or `**Status:** frozen record (YYYY-MM-DD)`; a Markdown file outside `documentation/` never
//! opens with `**Status:**`.

#[cfg(test)]
#[path = "tests/status_lines.rs"]
mod tests;

use anyhow::Result;

use crate::layout::{REFERENCE_FOLDER, STATUS_ARCHIVED, STATUS_FROZEN_PREFIX, STATUS_LIVE};
use crate::support::repository::{self, Repository};

use super::gate::Violation;

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for file in repository.files()? {
        if !repository::has_extension(file, "md")
            || file.starts_with(&format!("{REFERENCE_FOLDER}/"))
        {
            continue;
        }
        let text = repository.read_text(file)?;
        let first_line = text.lines().next().unwrap_or("");
        if file.starts_with("documentation/") {
            if !is_status_line(first_line) {
                violations.push(Violation::new(
                    file,
                    1,
                    "a document opens with `**Status:** live`, `**Status:** frozen record (YYYY-MM-DD)` or `**Status:** archived`",
                ));
            }
        } else if first_line.starts_with("**Status:**") {
            violations.push(Violation::new(
                file,
                1,
                "a code README or root file carries no status line",
            ));
        }
    }
    Ok(violations)
}

/// Whether the line is one of the three status lines.
pub(crate) fn is_status_line(line: &str) -> bool {
    if line == STATUS_LIVE || line == STATUS_ARCHIVED {
        return true;
    }
    let Some(date) = line
        .strip_prefix(STATUS_FROZEN_PREFIX)
        .and_then(|rest| rest.strip_suffix(')'))
    else {
        return false;
    };
    let digits_at = |range: std::ops::Range<usize>| {
        date.get(range)
            .is_some_and(|part| part.chars().all(|character| character.is_ascii_digit()))
    };
    date.len() == 10
        && digits_at(0..4)
        && date.get(4..5) == Some("-")
        && digits_at(5..7)
        && date.get(7..8) == Some("-")
        && digits_at(8..10)
}
