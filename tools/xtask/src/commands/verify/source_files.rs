//! The Rust sources the source gates walk, and the crate each belongs to.
//!
//! **Role:** lists the `.rs` files under crates and tools, tells test files from production
//! files, finds a file's crate folder and says whether a file is a crate root.
//!
//! **Position:** shared by `unsafe-allowlist`, `file-length`, `test-placement`, `module-headers`
//! and `crate-anatomy`.
//!
//! **Signals and state:** none; pure functions over the repository listing.
//!
//! **Invariants:** a test file is any file with a folder named `tests` on its path; a crate root
//! is `src/lib.rs` or `src/main.rs` directly in a crate folder.

use anyhow::{Result, bail};

use crate::layout::MEMBER_ROOTS;
use crate::support::repository::{self, Repository};

/// Every listed `.rs` file under crates and tools; an error when none of those folders holds
/// any file, because then the gate has nothing to check.
pub(crate) fn rust_files(repository: &Repository) -> Result<Vec<String>> {
    let files = repository.files()?;
    let under_roots: Vec<&String> = files
        .iter()
        .filter(|file| {
            MEMBER_ROOTS
                .iter()
                .any(|root| file.starts_with(&format!("{root}/")))
        })
        .collect();
    if under_roots.is_empty() {
        bail!("no source root: crates and tools hold no file");
    }
    Ok(under_roots
        .into_iter()
        .filter(|file| repository::has_extension(file, "rs"))
        .cloned()
        .collect())
}

/// Whether the file lies under a `tests` folder.
pub(crate) fn is_test_file(relative: &str) -> bool {
    repository::has_folder_named(relative, "tests")
}

/// The folder of the crate that owns the file: the nearest folder above it with a listed
/// `Cargo.toml`.
pub(crate) fn crate_folder_of<'path>(
    repository: &Repository,
    relative: &'path str,
) -> Result<Option<&'path str>> {
    let mut current = relative;
    while let Some((parent, _)) = current.rsplit_once('/') {
        if repository.has_file(&format!("{parent}/Cargo.toml"))? {
            return Ok(Some(parent));
        }
        current = parent;
    }
    Ok(None)
}

/// The package name of the crate that owns the file (its folder name), or the empty string.
pub(crate) fn crate_name_of<'path>(
    repository: &Repository,
    relative: &'path str,
) -> Result<&'path str> {
    Ok(crate_folder_of(repository, relative)?.map_or("", repository::file_name))
}

/// Whether the file is `src/lib.rs` or `src/main.rs` directly in its crate folder.
pub(crate) fn is_crate_root(repository: &Repository, relative: &str) -> Result<bool> {
    let Some(folder) = crate_folder_of(repository, relative)? else {
        return Ok(false);
    };
    Ok(relative == format!("{folder}/src/lib.rs") || relative == format!("{folder}/src/main.rs"))
}

/// The 1-based line of the first line containing `needle`, or 1 when none does.
pub(crate) fn line_of(text: &str, needle: &str) -> usize {
    text.lines()
        .position(|line| line.contains(needle))
        .map_or(1, |index| index + 1)
}

/// The 1-based line where a manifest declares the dependency `name` (`name.workspace = true` or
/// `name = …`), or 1 when it is not found.
pub(crate) fn dependency_line(manifest_text: &str, name: &str) -> usize {
    manifest_text
        .lines()
        .position(|line| {
            line.strip_prefix(name)
                .is_some_and(|rest| rest.starts_with('.') || rest.trim_start().starts_with('='))
        })
        .map_or(1, |index| index + 1)
}
