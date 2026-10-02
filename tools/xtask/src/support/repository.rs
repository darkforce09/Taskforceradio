//! The repository a command works on: its root folder and the files git would track there.
//!
//! **Role:** lists the repository's files once (tracked files plus untracked files that no ignore
//! rule hides, as `git ls-files --cached --others --exclude-standard` prints them), derives the
//! folders that hold them, and reads files by repository-relative path.
//!
//! **Position:** under every gate and command; the gates take a `Repository` so a test can point
//! one at a fixture tree in a temporary folder.
//!
//! **Signals and state:** the file list is computed on first use and cached; single-threaded.
//!
//! **Invariants:** paths use `/` and are relative to the root; the list is sorted and holds only
//! files that exist on disk; a failed `git` call is an error, never an empty list.

#[cfg(test)]
#[path = "tests/repository.rs"]
mod tests;

use std::cell::OnceCell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// A repository root and the files git lists under it.
pub(crate) struct Repository {
    root: PathBuf,
    files: OnceCell<Vec<String>>,
    folders: OnceCell<BTreeSet<String>>,
}

impl Repository {
    /// A repository rooted at `root`.
    pub(crate) fn at(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            files: OnceCell::new(),
            folders: OnceCell::new(),
        }
    }

    /// The repository this xtask binary was built from: two folders above `tools/xtask`.
    pub(crate) fn of_this_workspace() -> Result<Self> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let Some(root) = manifest_dir.ancestors().nth(2) else {
            bail!(
                "{} has no workspace root two levels up",
                manifest_dir.display()
            );
        };
        Ok(Self::at(root))
    }

    /// The root folder.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// The absolute path of a repository-relative path.
    pub(crate) fn absolute(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    /// Every listed file, sorted, as repository-relative paths.
    pub(crate) fn files(&self) -> Result<&[String]> {
        if let Some(files) = self.files.get() {
            return Ok(files);
        }
        let listed = self.list_files_with_git()?;
        Ok(self.files.get_or_init(|| listed))
    }

    /// Every folder that holds a listed file at any depth, as repository-relative paths without a
    /// trailing slash; the root itself is the empty string.
    pub(crate) fn folders(&self) -> Result<&BTreeSet<String>> {
        if let Some(folders) = self.folders.get() {
            return Ok(folders);
        }
        let mut folders = BTreeSet::new();
        folders.insert(String::new());
        for file in self.files()? {
            let mut current = file.as_str();
            while let Some((parent, _)) = current.rsplit_once('/') {
                folders.insert(parent.to_owned());
                current = parent;
            }
        }
        Ok(self.folders.get_or_init(|| folders))
    }

    /// Listed files under `prefix` (a folder path without a trailing slash).
    pub(crate) fn files_under(&self, prefix: &str) -> Result<Vec<&str>> {
        let with_slash = format!("{prefix}/");
        Ok(self
            .files()?
            .iter()
            .map(String::as_str)
            .filter(|file| file.starts_with(&with_slash))
            .collect())
    }

    /// Whether a listed file has exactly this path.
    pub(crate) fn has_file(&self, relative: &str) -> Result<bool> {
        Ok(self
            .files()?
            .binary_search_by(|file| file.as_str().cmp(relative))
            .is_ok())
    }

    /// Whether the path is a listed file, a folder holding listed files, or anything on disk.
    pub(crate) fn path_exists(&self, relative: &str) -> Result<bool> {
        let trimmed = relative.trim_end_matches('/');
        if self.has_file(trimmed)? || self.folders()?.contains(trimmed) {
            return Ok(true);
        }
        Ok(self.absolute(trimmed).symlink_metadata().is_ok())
    }

    /// The text of a repository file.
    pub(crate) fn read_text(&self, relative: &str) -> Result<String> {
        std::fs::read_to_string(self.absolute(relative))
            .with_context(|| format!("reading {relative}"))
    }

    fn list_files_with_git(&self) -> Result<Vec<String>> {
        let shell = xshell::Shell::new()?;
        shell.change_dir(&self.root);
        let output = xshell::cmd!(
            shell,
            "git ls-files -z --cached --others --exclude-standard"
        )
        .quiet()
        .ignore_status()
        .output()
        .context("running git ls-files")?;
        if !output.status.success() {
            bail!(
                "git ls-files failed in {}: {}",
                self.root.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        let mut files: Vec<String> = output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|entry| !entry.is_empty())
            .map(|entry| String::from_utf8_lossy(entry).into_owned())
            .filter(|entry| self.root.join(entry).symlink_metadata().is_ok())
            .collect();
        files.sort();
        files.dedup();
        Ok(files)
    }
}

/// The file name part of a repository-relative path.
pub(crate) fn file_name(relative: &str) -> &str {
    relative.rsplit_once('/').map_or(relative, |(_, name)| name)
}

/// The parent folder of a repository-relative path; the root is the empty string.
pub(crate) fn parent_folder(relative: &str) -> &str {
    relative.rsplit_once('/').map_or("", |(parent, _)| parent)
}

/// Whether any folder on the path is named `name`.
pub(crate) fn has_folder_named(relative: &str, name: &str) -> bool {
    let folders = parent_folder(relative);
    !folders.is_empty() && folders.split('/').any(|part| part == name)
}

/// Whether the path ends in `.<extension>`, compared without regard to case.
pub(crate) fn has_extension(relative: &str, extension: &str) -> bool {
    Path::new(relative)
        .extension()
        .is_some_and(|found| found.eq_ignore_ascii_case(extension))
}
