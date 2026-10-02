//! The `link-check` gate: links and backticked repository paths resolve, and no link climbs with
//! `../`.
//!
//! **Role:** reads every Markdown file outside `reference/`, checks each link target (and its
//! anchor) and each backticked string that names a repository path.
//!
//! **Position:** a repository gate; reads Markdown through `markdown` and existence through
//! [`Repository::path_exists`].
//!
//! **Signals and state:** none.
//!
//! **Invariants:** a backticked string is a repository path only when its first segment is apps,
//! crates, tools, documentation, assets or reference; such a path exists when it is a listed
//! file, a folder of listed files, or anything on disk (the ignored 1.0 reference tree included);
//! text inside code fences is never checked.

#[cfg(test)]
#[path = "tests/link_check.rs"]
mod tests;

use std::collections::BTreeMap;

use anyhow::Result;

use crate::layout::{LINKABLE_TOP_FOLDERS, REFERENCE_FOLDER};
use crate::support::markdown;
use crate::support::repository::{self, Repository};

use super::gate::Violation;

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    let mut anchor_cache: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in repository.files()? {
        if !repository::has_extension(file, "md")
            || file.starts_with(&format!("{REFERENCE_FOLDER}/"))
        {
            continue;
        }
        let text = repository.read_text(file)?;
        for line in markdown::lines(&text)
            .into_iter()
            .filter(|line| !line.in_fence)
        {
            for target in markdown::link_targets(line.text) {
                if let Some(problem) =
                    link_problem(repository, file, &text, &target, &mut anchor_cache)?
                {
                    violations.push(Violation::new(file, line.number, problem));
                }
            }
            for span in markdown::inline_code_spans(line.text) {
                let Some(path) = repository_path_in(span) else {
                    continue;
                };
                if !repository.path_exists(&path)? {
                    violations.push(Violation::new(
                        file,
                        line.number,
                        format!("the backticked path `{span}` does not exist; write a planned path as plain text"),
                    ));
                }
            }
        }
    }
    Ok(violations)
}

fn link_problem(
    repository: &Repository,
    file: &str,
    text: &str,
    target: &str,
    anchor_cache: &mut BTreeMap<String, Vec<String>>,
) -> Result<Option<String>> {
    if target.contains("://") || target.starts_with("mailto:") {
        return Ok(None);
    }
    if target.starts_with("../") || target.contains("/../") {
        return Ok(Some(format!(
            "`{target}` climbs with `../`; link from the repository root"
        )));
    }
    let (path_part, anchor) = target
        .split_once('#')
        .map_or((target, None), |(path, anchor)| (path, Some(anchor)));
    let resolved = if path_part.is_empty() {
        file.to_owned()
    } else if let Some(rooted) = path_part.strip_prefix('/') {
        rooted.trim_end_matches('/').to_owned()
    } else {
        let parent = repository::parent_folder(file);
        let relative = path_part.trim_start_matches("./").trim_end_matches('/');
        if parent.is_empty() {
            relative.to_owned()
        } else {
            format!("{parent}/{relative}")
        }
    };
    if !resolved.is_empty() && !repository.path_exists(&resolved)? {
        return Ok(Some(format!("the link target `{target}` does not exist")));
    }
    let Some(anchor) = anchor.filter(|anchor| !anchor.is_empty()) else {
        return Ok(None);
    };
    let target_file = if path_part.is_empty() {
        file.to_owned()
    } else if repository::has_extension(&resolved, "md") {
        resolved
    } else {
        return Ok(None);
    };
    if !anchor_cache.contains_key(&target_file) {
        let target_text = if target_file == file {
            text.to_owned()
        } else {
            repository.read_text(&target_file)?
        };
        anchor_cache.insert(target_file.clone(), markdown::heading_anchors(&target_text));
    }
    let anchors = anchor_cache
        .get(&target_file)
        .map_or(&[][..], Vec::as_slice);
    if anchors.iter().any(|known| known == anchor) {
        Ok(None)
    } else {
        Ok(Some(format!(
            "the anchor `#{anchor}` is not a heading of `{target_file}`"
        )))
    }
}

/// The repository path a backticked string names, or `None` when it does not look like one.
pub(crate) fn repository_path_in(span: &str) -> Option<String> {
    let candidate = span.strip_prefix('/').unwrap_or(span);
    let looks_like_text = candidate.is_empty()
        || candidate.contains(char::is_whitespace)
        || candidate.contains([
            '<', '>', '*', '?', '{', '}', '[', ']', '|', '"', '\'', '$', '…', '(', ')', '=', ',',
        ])
        || candidate.contains("...")
        || candidate.contains("::");
    if looks_like_text {
        return None;
    }
    let (first, _) = candidate.split_once('/')?;
    if !LINKABLE_TOP_FOLDERS.contains(&first) {
        return None;
    }
    let without_anchor = candidate
        .split_once('#')
        .map_or(candidate, |(path, _)| path);
    let without_line = strip_line_suffix(without_anchor);
    Some(without_line.trim_end_matches('/').to_owned())
}

/// `path.rs:14` and `path.rs:14-20` become `path.rs`.
fn strip_line_suffix(path: &str) -> &str {
    match path.rsplit_once(':') {
        Some((before, numbers))
            if !numbers.is_empty()
                && numbers
                    .chars()
                    .all(|character| character.is_ascii_digit() || character == '-') =>
        {
            before
        }
        _ => path,
    }
}
