//! The README span and the Contents block grammar of the README standard.
//!
//! **Role:** says which folders must carry a README.md, lists a folder's direct children, and
//! parses a README's `## Contents` `text` tree into its root line and entries.
//!
//! **Position:** used by `readme-coverage`, `readme-sections` and `new-crate`; reads through
//! [`Repository`].
//!
//! **Signals and state:** none; pure functions over the repository listing.
//!
//! **Invariants:** a folder is in the span when it lies at or under crates, tools,
//! documentation or assets and no folder on its path is `tests`, `target` or starts with `.`;
//! the Contents block is the first `text` fence between `## Contents` and the next `##` heading.

#[cfg(test)]
#[path = "tests/readme_span.rs"]
mod tests;

use anyhow::Result;

use crate::layout::{README_EXEMPT_FOLDERS, README_SPAN_ROOTS};
use crate::support::markdown;
use crate::support::repository::Repository;

/// Whether a folder (repository-relative, no trailing slash) must carry a README.md.
pub(crate) fn is_in_span(folder: &str) -> bool {
    let mut parts = folder.split('/');
    let Some(first) = parts.next() else {
        return false;
    };
    README_SPAN_ROOTS.contains(&first)
        && folder
            .split('/')
            .all(|part| !README_EXEMPT_FOLDERS.contains(&part) && !part.starts_with('.'))
}

/// Every folder in the span that holds a listed file, sorted.
pub(crate) fn span_folders(repository: &Repository) -> Result<Vec<String>> {
    Ok(repository
        .folders()?
        .iter()
        .filter(|folder| is_in_span(folder))
        .cloned()
        .collect())
}

/// The direct children of a folder: file names and folder names, each sorted.
#[derive(Debug, Default)]
pub(crate) struct FolderChildren {
    /// File names directly in the folder.
    pub(crate) files: Vec<String>,
    /// Names of folders directly in the folder that hold a listed file.
    pub(crate) folders: Vec<String>,
}

/// The direct children of `folder`.
pub(crate) fn children(repository: &Repository, folder: &str) -> Result<FolderChildren> {
    let mut result = FolderChildren::default();
    let prefix = if folder.is_empty() {
        String::new()
    } else {
        format!("{folder}/")
    };
    for file in repository.files()? {
        let Some(rest) = file.strip_prefix(&prefix) else {
            continue;
        };
        match rest.split_once('/') {
            None => result.files.push(rest.to_owned()),
            Some((child_folder, _)) => {
                if result
                    .folders
                    .last()
                    .is_none_or(|last| last != child_folder)
                {
                    result.folders.push(child_folder.to_owned());
                }
            }
        }
    }
    result.folders.sort();
    result.folders.dedup();
    Ok(result)
}

/// One entry line of a Contents block.
#[derive(Debug, Clone)]
pub(crate) struct ContentsEntry {
    /// The 1-based line number in the README.
    pub(crate) line: usize,
    /// The name or glob, without a trailing `/`.
    pub(crate) name: String,
    /// Whether the entry names a folder (it ends in `/`).
    pub(crate) is_folder: bool,
}

/// A parsed Contents block.
#[derive(Debug)]
pub(crate) struct ContentsBlock {
    /// The 1-based line number of the root line, and its text.
    pub(crate) root_line: Option<(usize, String)>,
    /// The entry lines that parsed.
    pub(crate) entries: Vec<ContentsEntry>,
    /// Lines that broke the grammar: their number and what is wrong.
    pub(crate) problems: Vec<(usize, String)>,
    /// The 1-based line number of the opening fence.
    pub(crate) opening_line: usize,
}

/// The Contents block of a README, or `None` when it has no `## Contents` heading or no `text`
/// fence under it.
pub(crate) fn contents_block(text: &str) -> Option<ContentsBlock> {
    let headings = markdown::headings(text);
    let contents_heading = headings
        .iter()
        .position(|heading| heading.level == 2 && heading.text == "Contents")?;
    let start = headings.get(contents_heading)?.line;
    let end = headings
        .iter()
        .skip(contents_heading + 1)
        .find(|heading| heading.level == 2)
        .map_or(usize::MAX, |heading| heading.line);
    let block = markdown::fenced_blocks(text).into_iter().find(|block| {
        block.info == "text" && block.opening_line > start && block.opening_line < end
    })?;
    let mut parsed = ContentsBlock {
        root_line: None,
        entries: Vec::new(),
        problems: Vec::new(),
        opening_line: block.opening_line,
    };
    let mut lines = block.lines.iter();
    if let Some((number, root)) = lines.next() {
        parsed.root_line = Some((*number, (*root).to_owned()));
    }
    for (number, line) in lines {
        match parse_entry(line) {
            Ok((name, is_folder)) => parsed.entries.push(ContentsEntry {
                line: *number,
                name,
                is_folder,
            }),
            Err(problem) => parsed.problems.push((*number, problem)),
        }
    }
    Some(parsed)
}

/// Parses one entry line into its name and whether it is a folder.
fn parse_entry(line: &str) -> std::result::Result<(String, bool), String> {
    if line.starts_with('│') || line.starts_with(' ') {
        return Err("a nested or indented entry; list only direct children".to_owned());
    }
    let body = line
        .strip_prefix("├── ")
        .or_else(|| line.strip_prefix("└── "))
        .unwrap_or(line);
    let Some((name, role)) = body.split_once("  ") else {
        return Err("an entry needs a name, two or more spaces and a role".to_owned());
    };
    let name = name.trim_end();
    if name.is_empty() || name.contains(' ') {
        return Err(format!("`{name}` is not a single name or glob"));
    }
    if role.trim().is_empty() {
        return Err(format!("the entry `{name}` has no role"));
    }
    Ok(match name.strip_suffix('/') {
        Some(folder) => (folder.to_owned(), true),
        None => (name.to_owned(), false),
    })
}
