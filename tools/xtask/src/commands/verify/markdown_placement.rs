//! The `markdown-placement` gate: Markdown only where it belongs, and live documents within 500
//! lines.
//!
//! **Role:** checks that every Markdown file outside `reference/` is under `documentation/`, a
//! README.md, a root CLAUDE.md, AGENTS.md or README.md, or a CREDITS.md under `assets/`; and that
//! no live document passes 500 lines.
//!
//! **Position:** a repository gate; the allowed names come from `layout`.
//!
//! **Signals and state:** none.
//!
//! **Invariants:** `reference/` is ignored; a live document is one whose first line is
//! `**Status:** live`.

#[cfg(test)]
#[path = "tests/markdown_placement.rs"]
mod tests;

use anyhow::Result;

use crate::layout::{
    ASSET_CREDITS_FILE, LIVE_DOCUMENT_MAX_LINES, REFERENCE_FOLDER, ROOT_MARKDOWN_FILES, STATUS_LIVE,
};
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
        let name = repository::file_name(file);
        let is_document = file.starts_with("documentation/");
        let allowed = is_document
            || name == "README.md"
            || (!file.contains('/') && ROOT_MARKDOWN_FILES.contains(&name))
            || (file.starts_with("assets/") && name == ASSET_CREDITS_FILE);
        if !allowed {
            violations.push(Violation::new(
                file,
                1,
                "Markdown lives under documentation/, as a README.md, or as the root CLAUDE.md, AGENTS.md or README.md",
            ));
            continue;
        }
        if is_document {
            let text = repository.read_text(file)?;
            let line_count = text.lines().count();
            if text.lines().next() == Some(STATUS_LIVE) && line_count > LIVE_DOCUMENT_MAX_LINES {
                violations.push(Violation::new(
                    file,
                    LIVE_DOCUMENT_MAX_LINES + 1,
                    format!("a live document stays within {LIVE_DOCUMENT_MAX_LINES} lines; this one has {line_count}: split it into a folder"),
                ));
            }
        }
    }
    Ok(violations)
}
