//! The `language-bans` gate: no scripts anywhere, and C/C++ only in the golden-vector oracle and
//! shim folders.
//!
//! **Role:** checks every listed file outside `reference/` for a banned extension (`.py`, `.sh`,
//! `.bash`, `.js`, `.ts`), a Makefile, or a C/C++ source outside the allowed folders.
//!
//! **Position:** a repository gate; the lists come from `layout`.
//!
//! **Signals and state:** none.
//!
//! **Invariants:** `reference/` is ignored; the check goes by file name only, never by content.

#[cfg(test)]
#[path = "tests/language_bans.rs"]
mod tests;

use anyhow::Result;

use crate::layout::{
    BANNED_EXTENSIONS, BANNED_FILE_NAMES, C_SOURCE_EXTENSIONS, C_SOURCE_FOLDERS, REFERENCE_FOLDER,
};
use crate::support::repository::{self, Repository};

use super::gate::Violation;

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for file in repository.files()? {
        if file.starts_with(&format!("{REFERENCE_FOLDER}/")) {
            continue;
        }
        let name = repository::file_name(file);
        let extension = name.rsplit_once('.').map(|(_, extension)| extension);
        if BANNED_FILE_NAMES.contains(&name)
            || extension.is_some_and(|extension| BANNED_EXTENSIONS.contains(&extension))
        {
            violations.push(Violation::new(
                file,
                1,
                "no Python, shell, Makefile or Node files: a repository task is a `cargo xtask` command",
            ));
        } else if extension.is_some_and(|extension| C_SOURCE_EXTENSIONS.contains(&extension))
            && !C_SOURCE_FOLDERS
                .iter()
                .any(|folder| file.starts_with(folder))
        {
            violations.push(Violation::new(
                file,
                1,
                format!(
                    "C and C++ sources live only in {}",
                    C_SOURCE_FOLDERS.join(" and ")
                ),
            ));
        }
    }
    Ok(violations)
}
