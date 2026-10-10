//! The `readme-sections` gate: each README's title, sections and Boundaries bullets, per kind.
//!
//! **Role:** reads the kind table and the legal heading order from the README standard, decides
//! each span folder's kind, and checks its README: an H1 title without backticks, `##` headings
//! that are legal, in order and within the kind's set, the required sections present, the three
//! Boundaries bullets, and no status line in a code README.
//!
//! **Position:** a repository gate; reads `documentation/standards/readme_standard.md`, the span
//! from `readme_span` and headings from `markdown`.
//!
//! **Signals and state:** none.
//!
//! **Invariants:** could not run when the standard's kind table or heading order is missing; the
//! kinds are picked in the standard's order (documentation folder, app, area root, crate root,
//! data, command-line, domain or subsystem, leaf), and a crate's `src/` is a domain.

#[cfg(test)]
#[path = "tests/readme_sections.rs"]
mod tests;

use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};

use crate::layout::{README_EXEMPT_FOLDERS, README_STANDARD, is_deliverable};
use crate::support::markdown;
use crate::support::readme_span;
use crate::support::repository::{self, Repository};

use super::gate::Violation;

const CORE_SECTIONS: &[&str] = &[
    "Contents",
    "How it works",
    "Boundaries",
    "Related documentation",
];
const BOUNDARY_BULLETS: &[&str] = &["- Depends on:", "- Used by:", "- Rules:"];
const KIND_NAMES: &[&str] = &[
    "documentation folder",
    "app",
    "area root",
    "crate root",
    "data",
    "command-line",
    "domain or subsystem",
    "leaf",
];

/// The kind table and the legal heading order, as the README standard states them.
pub(crate) struct ReadmeStandard {
    kinds: BTreeMap<String, Vec<String>>,
    order: Vec<String>,
}

impl ReadmeStandard {
    /// Parses the standard's `## Kinds` table and its "The legal `##` headings" sentence.
    pub(crate) fn parse(text: &str) -> Result<Self> {
        let mut kinds = BTreeMap::new();
        for line in text.lines().filter(|line| line.starts_with('|')) {
            let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
            let (Some(kind), Some(sections)) = (cells.first(), cells.get(1)) else {
                continue;
            };
            if KIND_NAMES.contains(kind) {
                let sections = if sections.eq_ignore_ascii_case("none") {
                    Vec::new()
                } else {
                    sections
                        .split(',')
                        .map(|section| section.trim().to_owned())
                        .collect()
                };
                kinds.insert((*kind).to_owned(), sections);
            }
        }
        for kind in KIND_NAMES {
            if !kinds.contains_key(*kind) {
                bail!("the kind table has no row for `{kind}`");
            }
        }
        let paragraph = text
            .split("\n\n")
            .find(|paragraph| paragraph.contains("The legal `##` headings"))
            .context("no \"The legal `##` headings\" sentence")?
            .replace('\n', " ");
        let listed = paragraph
            .split_once("are:")
            .context("the heading sentence has no `are:`")?
            .1;
        let listed = listed.split_once('.').map_or(listed, |(before, _)| before);
        let order: Vec<String> = listed
            .split(',')
            .map(|heading| heading.trim().to_owned())
            .collect();
        if order.len() < CORE_SECTIONS.len() {
            bail!("the heading order lists too few headings");
        }
        Ok(Self { kinds, order })
    }

    fn sections_of(&self, kind: &str) -> &[String] {
        self.kinds.get(kind).map_or(&[], Vec::as_slice)
    }
}

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let standard_text = repository
        .read_text(README_STANDARD)
        .with_context(|| format!("the README standard {README_STANDARD} is missing"))?;
    let standard = ReadmeStandard::parse(&standard_text)
        .with_context(|| format!("{README_STANDARD}: the kind table is unusable"))?;
    let mut violations = Vec::new();
    for folder in readme_span::span_folders(repository)? {
        let readme = format!("{folder}/README.md");
        if !repository.has_file(&readme)? {
            continue;
        }
        let text = repository.read_text(&readme)?;
        let kind = kind_of(repository, &folder, &text)?;
        violations.extend(readme_violations(
            repository, &standard, &folder, &readme, kind, &text,
        )?);
    }
    Ok(violations)
}

/// The README kind of a folder, picked in the standard's order.
fn kind_of(repository: &Repository, folder: &str, text: &str) -> Result<&'static str> {
    let parent = repository::parent_folder(folder);
    let headings = markdown::headings(text);
    let has_heading = |name: &str| {
        headings
            .iter()
            .any(|heading| heading.level == 2 && heading.text == name)
    };
    let kind = if folder == "documentation" || folder.starts_with("documentation/") {
        "documentation folder"
    } else if parent.starts_with("crates/") && is_deliverable(repository::file_name(folder)) {
        "app"
    } else if matches!(folder, "crates" | "tools" | "assets") || parent == "crates" {
        "area root"
    } else if repository.has_file(&format!("{folder}/Cargo.toml"))? {
        "crate root"
    } else if folder.starts_with("assets/") || has_heading("Format") {
        "data"
    } else if has_heading("Commands") {
        "command-line"
    } else if (repository::file_name(folder) == "src"
        && repository.has_file(&format!("{parent}/Cargo.toml"))?)
        || has_own_child_folders(repository, folder)?
    {
        "domain or subsystem"
    } else {
        "leaf"
    };
    Ok(kind)
}

fn has_own_child_folders(repository: &Repository, folder: &str) -> Result<bool> {
    Ok(readme_span::children(repository, folder)?
        .folders
        .iter()
        .any(|child| !README_EXEMPT_FOLDERS.contains(&child.as_str()) && !child.starts_with('.')))
}

fn readme_violations(
    repository: &Repository,
    standard: &ReadmeStandard,
    folder: &str,
    readme: &str,
    kind: &str,
    text: &str,
) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    let is_documentation = kind == "documentation folder";
    let first_line = text.lines().next().unwrap_or("");
    if !is_documentation && first_line.starts_with("**Status:**") {
        violations.push(Violation::new(
            readme,
            1,
            "a code README carries no status line",
        ));
    }
    let title = text
        .lines()
        .enumerate()
        .find(|(_, line)| !line.trim().is_empty() && !line.starts_with("**Status:**"));
    match title {
        Some((index, line)) if line.starts_with("# ") => {
            if line.contains('`') || line.trim_end().ends_with('/') {
                violations.push(Violation::new(
                    readme,
                    index + 1,
                    "the title is a plain name: no backticks, no path",
                ));
            }
        }
        _ => violations.push(Violation::new(
            readme,
            1,
            "the README opens with an `# ` H1 title",
        )),
    }
    let kind_sections = standard.sections_of(kind);
    let headings: Vec<_> = markdown::headings(text)
        .into_iter()
        .filter(|heading| heading.level == 2)
        .collect();
    let mut last_position: Option<usize> = None;
    for heading in &headings {
        let Some(position) = standard
            .order
            .iter()
            .position(|legal| legal == heading.text)
        else {
            violations.push(Violation::new(
                readme,
                heading.line,
                format!("`## {}` is not a README heading", heading.text),
            ));
            continue;
        };
        let in_kind = CORE_SECTIONS.contains(&heading.text)
            || kind_sections.iter().any(|section| section == heading.text);
        if !in_kind {
            violations.push(Violation::new(
                readme,
                heading.line,
                format!("`## {}` is not a section of the {kind} kind", heading.text),
            ));
        }
        if last_position.is_some_and(|last| position <= last) {
            violations.push(Violation::new(
                readme,
                heading.line,
                format!("`## {}` is out of order", heading.text),
            ));
        }
        last_position = Some(position);
    }
    let has = |name: &str| headings.iter().any(|heading| heading.text == name);
    let mut required: Vec<&str> = vec!["Contents"];
    if !may_skip_how_it_works(repository, folder)? {
        required.push("How it works");
    }
    required.extend(kind_sections.iter().map(String::as_str));
    required.push("Boundaries");
    for section in required {
        if !has(section) {
            violations.push(Violation::new(
                readme,
                1,
                format!("the {kind} kind needs a `## {section}` section"),
            ));
        }
    }
    violations.extend(boundaries_violations(readme, text));
    Ok(violations)
}

/// A folder with no child folders besides exempt ones and at most three files (README.md not
/// counted) may leave out How it works.
fn may_skip_how_it_works(repository: &Repository, folder: &str) -> Result<bool> {
    let children = readme_span::children(repository, folder)?;
    let file_count = children
        .files
        .iter()
        .filter(|name| *name != "README.md")
        .count();
    Ok(file_count <= 3 && !has_own_child_folders(repository, folder)?)
}

fn boundaries_violations(readme: &str, text: &str) -> Vec<Violation> {
    let lines = markdown::lines(text);
    let Some(start) = lines
        .iter()
        .position(|line| !line.in_fence && line.text.trim_end() == "## Boundaries")
    else {
        return Vec::new();
    };
    let heading_line = start + 1;
    let bullets: Vec<(usize, &str)> = lines
        .iter()
        .skip(start + 1)
        .take_while(|line| line.in_fence || !line.text.starts_with('#'))
        .filter(|line| !line.in_fence && line.text.starts_with("- "))
        .map(|line| (line.number, line.text))
        .collect();
    let mut violations = Vec::new();
    if bullets.len() != BOUNDARY_BULLETS.len() {
        violations.push(Violation::new(
            readme,
            heading_line,
            format!(
                "Boundaries holds exactly three top-level bullets; found {}",
                bullets.len()
            ),
        ));
    }
    for ((line, bullet), expected) in bullets.iter().zip(BOUNDARY_BULLETS) {
        if !bullet.starts_with(expected) {
            violations.push(Violation::new(
                readme,
                *line,
                format!("this Boundaries bullet must start with `{expected}`"),
            ));
        }
    }
    violations
}
