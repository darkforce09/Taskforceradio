//! [`Ts3::plugin_folder`]: the folder holding the plugin's files, from TeamSpeak or, when
//! TeamSpeak names none, from the loaded plugin library itself.
//!
//! **Role:** resolves the folder the plugin apps read `radio-sounds/` and `tfar_hrir/` from:
//! `getPluginPath` first, then the directory of the shared library this code is linked into,
//! found with `dladdr`. Logs which of the two answered.
//!
//! **Position:** a method group of `ts3_functions`, over `sound_and_paths` (`plugin_path`); called
//! by both plugin apps in `init`.
//!
//! **Signals and state:** no state; the loader's record of the library is read, not kept.
//!
//! **Invariants:** an answer is never an empty path; the `dladdr` answer is the canonical folder
//! of the object that holds [`loaded_library_folder`], which inside a plugin app is the plugin
//! `.so` TeamSpeak loaded from its plugins folder.

use std::path::PathBuf;

use super::handle::Ts3;
use crate::Ts3Error;

#[cfg(test)]
#[path = "tests/plugin_folder.rs"]
mod tests;

/// Where [`Ts3::plugin_folder`] found the folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PluginFolderSource {
    /// TeamSpeak's `getPluginPath`.
    TeamSpeak,
    /// The folder of the loaded plugin library, from `dladdr`.
    LoadedLibrary,
}

impl Ts3 {
    /// The folder holding the plugin's files: `getPluginPath`, or, when that fails or is empty,
    /// the folder of the loaded plugin library. Logs which one answered.
    ///
    /// # Errors
    ///
    /// The [`Ts3::plugin_path`] error when the library's folder cannot be found either.
    pub fn plugin_folder(&self) -> Result<PathBuf, Ts3Error> {
        self.plugin_folder_with_source().map(|(folder, _)| folder)
    }

    /// [`Ts3::plugin_folder`] together with where the folder came from.
    ///
    /// # Errors
    ///
    /// The [`Ts3::plugin_path`] error when the library's folder cannot be found either.
    pub(crate) fn plugin_folder_with_source(
        &self,
    ) -> Result<(PathBuf, PluginFolderSource), Ts3Error> {
        let teamspeak_error = match self.plugin_path() {
            Ok(folder) => {
                tracing::info!(folder = %folder.display(), "the plugin folder from getPluginPath");
                return Ok((folder, PluginFolderSource::TeamSpeak));
            }
            Err(error) => error,
        };
        if let Some(folder) = loaded_library_folder() {
            tracing::info!(
                folder = %folder.display(),
                %teamspeak_error,
                "the plugin folder from the loaded plugin library, as getPluginPath named none"
            );
            return Ok((folder, PluginFolderSource::LoadedLibrary));
        }
        Err(teamspeak_error)
    }
}

/// The canonical folder of the shared object this function is compiled into (the plugin `.so`
/// inside a plugin app; the test executable in this crate's tests), or `None` when the loader
/// knows no file for it.
#[cfg(unix)]
pub(crate) fn loaded_library_folder() -> Option<PathBuf> {
    use std::ffi::{CStr, OsStr, c_void};
    use std::os::unix::ffi::OsStrExt;

    let anchor: fn() -> Option<PathBuf> = loaded_library_folder;
    let mut info = libc::Dl_info {
        dli_fname: std::ptr::null(),
        dli_fbase: std::ptr::null_mut(),
        dli_sname: std::ptr::null(),
        dli_saddr: std::ptr::null_mut(),
    };
    // SAFETY: `anchor` is the address of a function in this object, so `dladdr` reads the
    // loader's map for it; `info` is a writable `Dl_info` that outlives the call.
    let found = unsafe { libc::dladdr(anchor as *const c_void, &raw mut info) };
    if found == 0 || info.dli_fname.is_null() {
        return None;
    }
    // SAFETY: on success `dli_fname` points at the NUL-terminated file name the dynamic loader
    // owns for this object, valid while the object stays loaded, which it does while this
    // function runs; the bytes are copied before returning.
    let file = unsafe { CStr::from_ptr(info.dli_fname) };
    let path = std::fs::canonicalize(OsStr::from_bytes(file.to_bytes())).ok()?;
    let folder = path.parent()?;
    if folder.as_os_str().is_empty() {
        return None;
    }
    Some(folder.to_path_buf())
}

/// No dynamic loader to ask outside Unix: the plugin folder comes from TeamSpeak alone.
#[cfg(not(unix))]
pub(crate) fn loaded_library_folder() -> Option<PathBuf> {
    None
}
