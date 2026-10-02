//! The `editorconfig` gate: the root `.editorconfig` rules on every listed text file outside
//! `reference/`.
//!
//! **Role:** checks UTF-8, LF line ends, a final newline, no trailing whitespace outside Markdown,
//! and space indentation in Rust, TOML, JSON and YAML files.
//!
//! **Position:** a repository gate; the file list comes from `git ls-files` through
//! [`Repository`], which is the "could not run" case.
//!
//! **Signals and state:** none.
//!
//! **Invariants:** a binary file (by extension, or holding a NUL byte) is skipped; Markdown keeps
//! its trailing spaces because two of them are a line break.

#[cfg(test)]
#[path = "tests/editorconfig.rs"]
mod tests;

use anyhow::{Context, Result};

use crate::layout::REFERENCE_FOLDER;
use crate::support::repository::{self, Repository};

use super::gate::Violation;

const BINARY_EXTENSIONS: &[&str] = &[
    "wav", "paa", "dll", "so", "exe", "bin", "png", "jpg", "jpeg", "gif", "ico", "pbo", "psd",
    "zip", "db", "sqlite",
];
const SPACE_INDENTED_EXTENSIONS: &[&str] = &["rs", "toml", "json", "yml", "yaml"];

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for file in repository.files()? {
        if file.starts_with(&format!("{REFERENCE_FOLDER}/")) {
            continue;
        }
        let extension = repository::file_name(file)
            .rsplit_once('.')
            .map(|(_, extension)| extension.to_ascii_lowercase());
        if extension
            .as_deref()
            .is_some_and(|extension| BINARY_EXTENSIONS.contains(&extension))
        {
            continue;
        }
        let path = repository.absolute(file);
        if path.is_dir() {
            continue;
        }
        let bytes = std::fs::read(&path).with_context(|| format!("reading {file}"))?;
        if bytes.contains(&0) {
            continue;
        }
        violations.extend(text_file_violations(file, extension.as_deref(), &bytes));
    }
    Ok(violations)
}

/// Every rule a text file breaks.
pub(crate) fn text_file_violations(
    file: &str,
    extension: Option<&str>,
    bytes: &[u8],
) -> Vec<Violation> {
    let mut violations = Vec::new();
    let Ok(text) = std::str::from_utf8(bytes) else {
        violations.push(Violation::new(file, 1, "the file is not UTF-8"));
        return violations;
    };
    if let Some(index) = text.split('\n').position(|line| line.ends_with('\r')) {
        violations.push(Violation::new(
            file,
            index + 1,
            "CRLF line end; the repository uses LF",
        ));
    }
    if !text.is_empty() && !text.ends_with('\n') {
        violations.push(Violation::new(
            file,
            text.lines().count().max(1),
            "no final newline",
        ));
    }
    let is_markdown = extension == Some("md");
    let is_space_indented =
        extension.is_some_and(|extension| SPACE_INDENTED_EXTENSIONS.contains(&extension));
    for (index, line) in text.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if !is_markdown && (line.ends_with(' ') || line.ends_with('\t')) {
            violations.push(Violation::new(file, index + 1, "trailing whitespace"));
        }
        if is_space_indented
            && line
                .chars()
                .take_while(|character| character.is_whitespace())
                .any(|character| character == '\t')
        {
            violations.push(Violation::new(
                file,
                index + 1,
                "tab indentation; this file type indents with spaces",
            ));
        }
    }
    violations
}
