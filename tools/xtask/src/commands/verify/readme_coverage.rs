//! The `readme-coverage` gate: a README.md in every span folder, whose Contents lists the folder
//! exactly.
//!
//! **Role:** walks every folder in the README span, checks it has a README.md, parses its
//! Contents block and matches the entries against the folder's direct children.
//!
//! **Position:** a repository gate; the span and the Contents grammar come from `readme_span`,
//! globs from `glob_pattern`.
//!
//! **Signals and state:** none.
//!
//! **Invariants:** every direct child except README.md matches exactly one entry of its own kind
//! (file or folder), and every entry matches at least one child; the root line is the folder's
//! repository path followed by `/`.

#[cfg(test)]
#[path = "tests/readme_coverage.rs"]
mod tests;

use anyhow::Result;

use crate::support::glob_pattern;
use crate::support::readme_span::{self, ContentsBlock, ContentsEntry};
use crate::support::repository::Repository;

use super::gate::Violation;

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for folder in readme_span::span_folders(repository)? {
        let readme = format!("{folder}/README.md");
        if !repository.has_file(&readme)? {
            violations.push(Violation::new(
                format!("{folder}/"),
                1,
                "this folder has no README.md",
            ));
            continue;
        }
        let text = repository.read_text(&readme)?;
        let Some(block) = readme_span::contents_block(&text) else {
            violations.push(Violation::new(
                &readme,
                1,
                "no `## Contents` heading with a `text` block under it",
            ));
            continue;
        };
        check_block(repository, &folder, &readme, &block, &mut violations)?;
    }
    Ok(violations)
}

fn check_block(
    repository: &Repository,
    folder: &str,
    readme: &str,
    block: &ContentsBlock,
    violations: &mut Vec<Violation>,
) -> Result<()> {
    let expected_root = format!("{folder}/");
    match &block.root_line {
        Some((_, root)) if root.trim_end() == expected_root => {}
        Some((line, root)) => violations.push(Violation::new(
            readme,
            *line,
            format!("the Contents root line is `{root}`; expected `{expected_root}`"),
        )),
        None => violations.push(Violation::new(
            readme,
            block.opening_line,
            "the Contents block is empty",
        )),
    }
    for (line, problem) in &block.problems {
        violations.push(Violation::new(readme, *line, problem.clone()));
    }
    let children = readme_span::children(repository, folder)?;
    let files: Vec<&String> = children
        .files
        .iter()
        .filter(|name| *name != "README.md")
        .collect();
    let file_entries: Vec<&ContentsEntry> = block
        .entries
        .iter()
        .filter(|entry| !entry.is_folder)
        .collect();
    let folder_entries: Vec<&ContentsEntry> = block
        .entries
        .iter()
        .filter(|entry| entry.is_folder)
        .collect();
    match_children(readme, block, &files, &file_entries, "file", violations);
    let folders: Vec<&String> = children.folders.iter().collect();
    match_children(
        readme,
        block,
        &folders,
        &folder_entries,
        "folder",
        violations,
    );
    Ok(())
}

fn entry_matches(entry: &ContentsEntry, name: &str) -> bool {
    if glob_pattern::is_glob(&entry.name) {
        glob_pattern::matches(&entry.name, name)
    } else {
        entry.name == name
    }
}

fn match_children(
    readme: &str,
    block: &ContentsBlock,
    children: &[&String],
    entries: &[&ContentsEntry],
    kind: &str,
    violations: &mut Vec<Violation>,
) {
    let suffix = if kind == "folder" { "/" } else { "" };
    for child in children {
        let matching = entries
            .iter()
            .filter(|entry| entry_matches(entry, child))
            .count();
        if matching == 0 {
            violations.push(Violation::new(
                readme,
                block.opening_line,
                format!("the {kind} `{child}{suffix}` is not listed in Contents"),
            ));
        } else if matching > 1 {
            violations.push(Violation::new(
                readme,
                block.opening_line,
                format!("the {kind} `{child}{suffix}` matches {matching} Contents entries; it must match one"),
            ));
        }
    }
    for entry in entries {
        if !children.iter().any(|child| entry_matches(entry, child)) {
            violations.push(Violation::new(
                readme,
                entry.line,
                format!(
                    "the entry `{}{suffix}` names no {kind} in this folder",
                    entry.name
                ),
            ));
        }
    }
}
