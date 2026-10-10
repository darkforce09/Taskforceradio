//! The `unsafe-allowlist` gate: where `unsafe_code` may be lifted, and a `// SAFETY:` comment
//! above every unsafe block.
//!
//! **Role:** finds every `allow(unsafe_code)` attribute and every `unsafe {` block in the Rust
//! sources under crates and tools, and checks each against the allow-list and the comment
//! rules.
//!
//! **Position:** a repository gate; reads sources through `rust_source` and the allow-list from
//! `layout`.
//!
//! **Signals and state:** none.
//!
//! **Invariants:** a lift is legal only as `#![allow(unsafe_code)]` at the crate root of an
//! allow-listed crate, with a plain `//` reason comment on the line directly above; an unsafe
//! block needs `SAFETY:` in the comment lines directly above it.

#[cfg(test)]
#[path = "tests/unsafe_allowlist.rs"]
mod tests;

use anyhow::Result;

use crate::layout::UNSAFE_ALLOWED;
use crate::support::repository::Repository;
use crate::support::rust_source;

use super::gate::Violation;
use super::source_files::{crate_name_of, is_crate_root, rust_files};

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for file in rust_files(repository)? {
        let text = repository.read_text(&file)?;
        let raw_lines: Vec<&str> = text.lines().collect();
        let code_lines = rust_source::code_lines(&text);
        for (start, attribute) in attributes(&code_lines) {
            if !(attribute.contains("allow") && attribute.contains("unsafe_code")) {
                continue;
            }
            let crate_name = crate_name_of(repository, &file)?;
            if !UNSAFE_ALLOWED.contains(&crate_name) {
                violations.push(Violation::new(
                    &file,
                    start + 1,
                    format!("`{crate_name}` may not lift `unsafe_code`: it is not on the unsafe allow-list"),
                ));
                continue;
            }
            if !attribute.starts_with("#![") || !is_crate_root(repository, &file)? {
                violations.push(Violation::new(
                    &file,
                    start + 1,
                    "lift `unsafe_code` only with `#![allow(unsafe_code)]` at the crate root",
                ));
            }
            let reason_above = start
                .checked_sub(1)
                .and_then(|above| raw_lines.get(above))
                .map(|line| line.trim())
                .is_some_and(|line| {
                    line.starts_with("//") && !line.starts_with("//!") && line.len() > 2
                });
            if !reason_above {
                violations.push(Violation::new(
                    &file,
                    start + 1,
                    "`#![allow(unsafe_code)]` needs a reason comment on the line directly above",
                ));
            }
        }
        for (index, code) in code_lines.iter().enumerate() {
            if !(code.contains("unsafe {") || code.contains("unsafe{")) {
                continue;
            }
            if !has_safety_comment_above(&raw_lines, index)
                && !has_safety_comment_above(
                    &raw_lines,
                    statement_start(&code_lines, &raw_lines, index),
                )
            {
                violations.push(Violation::new(
                    &file,
                    index + 1,
                    "an `unsafe` block needs a `// SAFETY:` comment directly above it",
                ));
            }
        }
    }
    Ok(violations)
}

/// Every attribute in the code view: its 0-based first line and its text joined across lines.
fn attributes(code_lines: &[String]) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    let mut current: Option<(usize, String)> = None;
    for (index, line) in code_lines.iter().enumerate() {
        let trimmed = line.trim();
        let (start, mut text) = match current.take() {
            Some(open) => open,
            None if trimmed.starts_with("#[") || trimmed.starts_with("#![") => {
                (index, String::new())
            }
            None => continue,
        };
        text.push_str(trimmed);
        if text.matches('[').count() <= text.matches(']').count() {
            found.push((start, text));
        } else {
            current = Some((start, text));
        }
    }
    found
}

/// Whether the comment lines directly above line `index` hold `SAFETY:`.
fn has_safety_comment_above(raw_lines: &[&str], index: usize) -> bool {
    let mut cursor = index;
    while let Some(above) = cursor.checked_sub(1) {
        let Some(line) = raw_lines.get(above) else {
            return false;
        };
        if !rust_source::is_comment_line(line) {
            return false;
        }
        if line.contains("SAFETY:") {
            return true;
        }
        cursor = above;
    }
    false
}

/// The first line of the statement that line `index` continues: lines above are part of it while
/// they are code that ends in neither `;`, `{`, `}` nor `,` (a `let x =` above an unsafe block).
fn statement_start(code_lines: &[String], raw_lines: &[&str], index: usize) -> usize {
    let mut start = index;
    while let Some(above) = start.checked_sub(1) {
        let code = code_lines.get(above).map_or("", |line| line.trim());
        let is_comment = raw_lines
            .get(above)
            .is_some_and(|line| rust_source::is_comment_line(line));
        if code.is_empty() || is_comment || code.ends_with([';', '{', '}', ',']) {
            break;
        }
        start = above;
    }
    start
}
