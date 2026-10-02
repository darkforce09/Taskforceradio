//! [`TemporaryFolder`]: a fresh folder under the system temp folder that lives as long as a host
//! session, for the plugin and config paths the fakes answer with.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A uniquely named folder under [`std::env::temp_dir`], removed with its contents on drop.
#[derive(Debug)]
pub(crate) struct TemporaryFolder {
    /// The folder's path.
    path: PathBuf,
}

impl TemporaryFolder {
    /// Creates `<temp>/<prefix>-<process id>-<counter>`.
    pub(crate) fn create(prefix: &str) -> std::io::Result<Self> {
        static NEXT_FOLDER: AtomicUsize = AtomicUsize::new(0);
        let name = format!(
            "{prefix}-{}-{}",
            std::process::id(),
            NEXT_FOLDER.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(name);
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
        // A folder that cannot be removed is left for the system's temp cleaning; a drop never
        // fails.
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
