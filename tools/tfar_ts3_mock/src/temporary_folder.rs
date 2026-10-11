//! [`TemporaryFolder`]: a fresh folder under the repository's `target/tmp/` that lives as long as a
//! host session, for the plugin and config paths the fakes answer with.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The folder temporary folders are made in: `target/tmp/` of this checkout, never the system
/// temp folder.
pub(crate) fn temporary_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp")
}

/// A uniquely named folder under [`temporary_root`], removed with its contents on drop.
#[derive(Debug)]
pub(crate) struct TemporaryFolder {
    /// The folder's path.
    path: PathBuf,
}

impl TemporaryFolder {
    /// Creates `target/tmp/<prefix>-<process id>-<counter>`.
    pub(crate) fn create(prefix: &str) -> std::io::Result<Self> {
        static NEXT_FOLDER: AtomicUsize = AtomicUsize::new(0);
        let name = format!(
            "{prefix}-{}-{}",
            std::process::id(),
            NEXT_FOLDER.fetch_add(1, Ordering::Relaxed)
        );
        let path = temporary_root().join(name);
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    /// The folder's path.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryFolder {
    fn drop(&mut self) {
        // A folder that cannot be removed is left behind in `target/tmp/`; a drop never
        // fails.
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
