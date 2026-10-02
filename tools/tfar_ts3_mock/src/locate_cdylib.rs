//! [`locate_cdylib`]: finds the plugin library a crate's build produced, from inside that crate's
//! test executable.
//!
//! **Role:** lets a plugin app's integration test load its own `cdylib` (`lib<crate>.so`) into
//! the mock without knowing the target folder, profile or build directory.
//!
//! **Position:** called by the plugin apps' integration tests before
//! [`MockHost::load`](crate::MockHost::load).
//!
//! **Signals and state:** reads the running executable's path and the file system; no state.
//!
//! **Invariants:** looks only in the executable's folder and the two folders above it, each
//! itself and its `deps/` (cargo's `target-*/debug/deps/` and the `debug/` it uplifts to);
//! answers the first file found, or `None`.

use std::path::{Path, PathBuf};

#[cfg(test)]
#[path = "tests/locate_cdylib.rs"]
mod tests;

/// How many folders above the executable are searched.
const SEARCHED_FOLDER_LEVELS: usize = 3;

/// The `cdylib` file of `crate_name` (`lib<crate_name>.so` on Linux) built beside or above the
/// running executable, with `-` in the name read as `_`.
pub fn locate_cdylib(crate_name: &str) -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    locate_cdylib_from(&executable, crate_name)
}

/// [`locate_cdylib`] for the executable at `executable`.
pub(crate) fn locate_cdylib_from(executable: &Path, crate_name: &str) -> Option<PathBuf> {
    let file_name = cdylib_file_name(crate_name);
    executable
        .ancestors()
        .skip(1)
        .take(SEARCHED_FOLDER_LEVELS)
        .flat_map(|folder| {
            [
                folder.join(&file_name),
                folder.join("deps").join(&file_name),
            ]
        })
        .find(|candidate| candidate.is_file())
}

/// The platform's file name of the `cdylib` of `crate_name`.
pub(crate) fn cdylib_file_name(crate_name: &str) -> String {
    format!(
        "{}{}{}",
        std::env::consts::DLL_PREFIX,
        crate_name.replace('-', "_"),
        std::env::consts::DLL_SUFFIX
    )
}
