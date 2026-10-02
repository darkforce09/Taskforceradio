//! The `reference-untouched` gate: every file under `reference/` against the reference manifest.
//!
//! **Role:** recomputes the sha256 of every reference file (minus the README, the manifest and the
//! 1.0 tree's `.git` folder) and reports each added, removed and changed file.
//!
//! **Position:** a repository gate; hashing and the manifest format come from `reference_hashes`.
//!
//! **Signals and state:** none; reads only.
//!
//! **Invariants:** could not run when the manifest is missing or unreadable; the gate never writes
//! anything under `reference/`.

#[cfg(test)]
#[path = "tests/reference_untouched.rs"]
mod tests;

use anyhow::{Context, Result};

use crate::layout::{REFERENCE_FOLDER, REFERENCE_MANIFEST};
use crate::support::reference_hashes;
use crate::support::repository::Repository;

use super::gate::Violation;

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let manifest_text = repository
        .read_text(REFERENCE_MANIFEST)
        .with_context(|| format!("the manifest {REFERENCE_MANIFEST} is missing or unreadable"))?;
    let expected = reference_hashes::parse(&manifest_text)
        .with_context(|| format!("{REFERENCE_MANIFEST} is malformed"))?;
    let actual = reference_hashes::hash_tree(&repository.absolute(REFERENCE_FOLDER))?;
    let mut violations = Vec::new();
    for (path, hash) in &expected {
        match actual.get(path) {
            None => violations.push(Violation::new(
                format!("{REFERENCE_FOLDER}/{path}"),
                1,
                "removed: listed in the manifest but missing",
            )),
            Some(actual_hash) if actual_hash != hash => violations.push(Violation::new(
                format!("{REFERENCE_FOLDER}/{path}"),
                1,
                format!("changed: sha256 {actual_hash}, the manifest says {hash}"),
            )),
            Some(_) => {}
        }
    }
    for path in actual.keys().filter(|path| !expected.contains_key(*path)) {
        violations.push(Violation::new(
            format!("{REFERENCE_FOLDER}/{path}"),
            1,
            "added: not in the manifest",
        ));
    }
    Ok(violations)
}
