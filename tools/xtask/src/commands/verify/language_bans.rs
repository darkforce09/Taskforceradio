//! The `language-bans` gate: no scripts anywhere, C/C++ only in the golden-vector oracle and
//! shim folders, and `.h` reference headers also in the header-only folders.
//!
//! **Role:** checks every listed file outside `reference/` for a banned extension (`.py`, `.sh`,
//! `.bash`, `.js`, `.ts`), a Makefile, or a C/C++ file outside the folders `layout` allows for
//! it.
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
    BANNED_EXTENSIONS, BANNED_FILE_NAMES, C_HEADER_ONLY_FOLDERS, C_SOURCE_EXTENSIONS,
    C_SOURCE_FOLDERS, REFERENCE_FOLDER, c_file_allowed,
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
        } else if let Some(extension) = extension
            && C_SOURCE_EXTENSIONS.contains(&extension)
            && !c_file_allowed(file, extension)
        {
            violations.push(Violation::new(
                file,
                1,
                format!(
                    "C and C++ sources live only in {}; `.h` reference headers also in {}",
                    C_SOURCE_FOLDERS.join(" and "),
                    C_HEADER_ONLY_FOLDERS.join(" and ")
                ),
            ));
        }
    }
    Ok(violations)
}
