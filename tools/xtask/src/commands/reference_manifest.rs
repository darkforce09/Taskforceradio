//! `cargo xtask reference-manifest`: writes `reference/MANIFEST.sha256`, the checksum list
//! `reference-untouched` compares against.
//!
//! **Role:** hashes the reference trees and writes the sorted `sha256  path` list, refusing to
//! overwrite an existing manifest unless given `--force` with a reason.
//!
//! **Position:** called by `main`; hashing comes from `reference_hashes`.
//!
//! **Signals and state:** none; writes exactly one file.
//!
//! **Invariants:** the only file it writes is the manifest; an existing manifest survives unless
//! `--force` comes with a non-empty `--reason`.

#[cfg(test)]
#[path = "tests/reference_manifest.rs"]
mod tests;

use anyhow::{Context, Result, bail};

use crate::layout::{REFERENCE_FOLDER, REFERENCE_MANIFEST};
use crate::support::reference_hashes;
use crate::support::repository::Repository;

/// Writes the manifest; returns how many files it lists.
pub(crate) fn run(repository: &Repository, force_reason: Option<&str>) -> Result<usize> {
    let manifest = repository.absolute(REFERENCE_MANIFEST);
    if manifest.exists() {
        match force_reason.map(str::trim) {
            None => bail!(
                "{REFERENCE_MANIFEST} exists; pass `--force --reason \"<why>\"` to overwrite it"
            ),
            Some("") => bail!("`--force` needs a non-empty `--reason`"),
            Some(reason) => println!("overwriting {REFERENCE_MANIFEST}: {reason}"),
        }
    }
    let hashes = reference_hashes::hash_tree(&repository.absolute(REFERENCE_FOLDER))?;
    std::fs::write(&manifest, reference_hashes::render(&hashes))
        .with_context(|| format!("writing {REFERENCE_MANIFEST}"))?;
    println!("wrote {REFERENCE_MANIFEST}: {} files", hashes.len());
    Ok(hashes.len())
}
