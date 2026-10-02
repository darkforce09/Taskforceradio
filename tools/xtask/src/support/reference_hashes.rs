//! The sha256 list of the reference trees: computing it, writing it and reading it back.
//!
//! **Role:** hashes every file under `reference/` except the README, the manifest itself and the
//! 1.0 tree's `.git` folder, and reads and writes the manifest in `sha256sum` format.
//!
//! **Position:** used by `reference-manifest` (writes) and `reference-untouched` (compares).
//!
//! **Signals and state:** none; reads files, never writes under `reference/` itself.
//!
//! **Invariants:** manifest lines are `<64 hex>  <path relative to reference/>`, sorted by path;
//! the list read back from a rendered manifest equals the list rendered.

#[cfg(test)]
#[path = "tests/reference_hashes.rs"]
mod tests;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Read as _;
use std::path::Path;

use anyhow::{Context, Result, bail};
use sha2::{Digest as _, Sha256};

use crate::layout::REFERENCE_EXCLUDED;

/// Path relative to `reference/` mapped to the lowercase hex sha256 of the file.
pub(crate) type HashList = BTreeMap<String, String>;

/// Whether a path relative to `reference/` is left out of the manifest.
pub(crate) fn is_excluded(relative: &str) -> bool {
    REFERENCE_EXCLUDED
        .iter()
        .any(|excluded| match excluded.strip_suffix('/') {
            Some(folder) => relative == folder || relative.starts_with(excluded),
            None => relative == *excluded,
        })
}

/// Hashes every file under `reference_folder`, minus the excluded paths.
pub(crate) fn hash_tree(reference_folder: &Path) -> Result<HashList> {
    if !reference_folder.is_dir() {
        bail!("{} is not a folder", reference_folder.display());
    }
    let mut hashes = HashList::new();
    let walker = walkdir::WalkDir::new(reference_folder)
        .follow_links(false)
        .sort_by_file_name();
    for entry in walker {
        let entry = entry.with_context(|| format!("walking {}", reference_folder.display()))?;
        if entry.file_type().is_dir() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(reference_folder)
            .context("a walked path left the reference folder")?
            .to_string_lossy()
            .replace('\\', "/");
        if is_excluded(&relative) {
            continue;
        }
        hashes.insert(relative, hash_file(entry.path())?);
    }
    Ok(hashes)
}

/// The lowercase hex sha256 of one file.
pub(crate) fn hash_file(path: &Path) -> Result<String> {
    let mut file =
        std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1 << 16];
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("reading {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(buffer.get(..read).unwrap_or(&[]));
    }
    let mut hex = String::with_capacity(64);
    for byte in &hasher.finalize() {
        let _ = write!(hex, "{byte:02x}");
    }
    Ok(hex)
}

/// The manifest text: one `<hash>  <path>` line per file, sorted by path.
pub(crate) fn render(hashes: &HashList) -> String {
    let mut text = String::new();
    for (path, hash) in hashes {
        let _ = writeln!(text, "{hash}  {path}");
    }
    text
}

/// Reads a manifest back; a malformed line is an error naming its line number.
pub(crate) fn parse(text: &str) -> Result<HashList> {
    let mut hashes = HashList::new();
    for (index, line) in text.lines().enumerate() {
        let Some((hash, path)) = line.split_once("  ") else {
            bail!("line {} is not `<sha256>  <path>`", index + 1);
        };
        if hash.len() != 64 || !hash.chars().all(|character| character.is_ascii_hexdigit()) {
            bail!("line {} has no 64-digit sha256", index + 1);
        }
        hashes.insert(path.to_owned(), hash.to_owned());
    }
    Ok(hashes)
}
