//! [`PluginLibrary`]: a TeamSpeak plugin `.so` opened with `libloading`, its exports resolved and
//! its metadata read, not yet started.
//!
//! **Role:** what TeamSpeak does when it lists a plugin: load the file, find the exports, read
//! name, version, API version, author, description and the optional answers.
//!
//! **Position:** opened by [`MockHost::load`](crate::MockHost::load) (or by a caller that wants
//! the metadata first) and handed to [`MockHost::start`](crate::MockHost::start).
//!
//! **Signals and state:** the loaded library and the copied metadata; no fake state is touched.
//!
//! **Invariants:** the library outlives every export pointer taken from it (they live in the same
//! value, and the library is its last field); metadata strings are copied, so no pointer into the
//! plugin is kept.

use std::ffi::{CStr, c_char};
use std::path::{Path, PathBuf};

use libloading::Library;

use super::plugin_exports::{NumberExport, PluginExports, TextExport, resolve_exports};
use crate::{Error, Result};

/// What a plugin says about itself before it is started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginMetadata {
    /// `ts3plugin_name`.
    pub name: String,
    /// `ts3plugin_version`.
    pub version: String,
    /// `ts3plugin_apiVersion`.
    pub api_version: i32,
    /// `ts3plugin_author`.
    pub author: String,
    /// `ts3plugin_description`.
    pub description: String,
    /// `ts3plugin_requestAutoload` answered non-zero; `false` when the export is absent.
    pub requests_autoload: bool,
    /// `ts3plugin_offersConfigure`, when exported.
    pub offers_configure: Option<i32>,
    /// `ts3plugin_commandKeyword`, when exported and non-null.
    pub command_keyword: Option<String>,
    /// `ts3plugin_infoTitle`, when exported and non-null.
    pub info_title: Option<String>,
}

/// A loaded plugin library with its exports and metadata.
#[derive(Debug)]
pub struct PluginLibrary {
    /// The file it was loaded from.
    path: PathBuf,
    /// The resolved exports.
    exports: PluginExports,
    /// The metadata read at open.
    metadata: PluginMetadata,
    /// The loaded library; the last field, so it is unloaded after everything else is dropped.
    library: Library,
}

impl PluginLibrary {
    /// Loads the plugin at `path`, resolves its exports and reads its metadata.
    ///
    /// # Errors
    ///
    /// [`Error::LoadLibrary`] when the file cannot be loaded; [`Error::MissingRequiredSymbol`]
    /// when a required export (`ts3plugin_name` … `ts3plugin_shutdown`) is absent.
    pub fn open(path: &Path) -> Result<Self> {
        // SAFETY: loading runs the library's initialisers; a TeamSpeak plugin is built to be
        // loaded into a host process, which is what the mock is.
        let library = unsafe { Library::new(path) }.map_err(|source| Error::LoadLibrary {
            path: path.to_owned(),
            source,
        })?;
        // SAFETY: a file loaded as a TeamSpeak plugin exports its `ts3plugin_*` symbols with the
        // SDK's API 26 signatures, which `PluginExports` spells.
        let exports = unsafe { resolve_exports(&library) }.map_err(|(symbol, source)| {
            Error::MissingRequiredSymbol {
                path: path.to_owned(),
                symbol,
                source,
            }
        })?;
        let metadata = read_metadata(&exports);
        Ok(Self {
            path: path.to_owned(),
            exports,
            metadata,
            library,
        })
    }

    /// A library from parts, for tests that host an in-process plugin.
    #[cfg(test)]
    pub(crate) fn from_parts(path: PathBuf, exports: &PluginExports, library: Library) -> Self {
        let metadata = read_metadata(exports);
        Self {
            path,
            exports: *exports,
            metadata,
            library,
        }
    }

    /// The file the plugin was loaded from.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The resolved exports, valid while `self` lives.
    pub fn exports(&self) -> &PluginExports {
        &self.exports
    }

    /// The metadata read at open.
    pub fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    /// The loaded library, for symbols beyond the SDK's.
    pub fn library(&self) -> &Library {
        &self.library
    }
}

/// Calls the metadata exports and copies their answers.
fn read_metadata(exports: &PluginExports) -> PluginMetadata {
    PluginMetadata {
        name: call_text(exports.name),
        version: call_text(exports.version),
        api_version: call_number(exports.api_version),
        author: call_text(exports.author),
        description: call_text(exports.description),
        requests_autoload: exports
            .request_autoload
            .is_some_and(|export| call_number(export) != 0),
        offers_configure: exports.offers_configure.map(call_number),
        command_keyword: exports.command_keyword.and_then(call_optional_text),
        info_title: exports.info_title.and_then(call_optional_text),
    }
}

/// The answer of a text export, copied; null reads as `""`.
fn call_text(export: TextExport) -> String {
    call_optional_text(export).unwrap_or_default()
}

/// The answer of a text export, copied; `None` for null.
fn call_optional_text(export: TextExport) -> Option<String> {
    // SAFETY: a text export takes no argument and returns a static string (or null) of the
    // loaded library, which `exports` does not outlive.
    let text = unsafe { export() };
    (!text.is_null()).then(|| copied_text(text))
}

/// The answer of a number export.
fn call_number(export: NumberExport) -> i32 {
    // SAFETY: a number export takes no argument and only returns a value.
    unsafe { export() }
}

/// The C string at `text`, lossily as UTF-8; null reads as `""`.
pub(crate) fn copied_text(text: *const c_char) -> String {
    if text.is_null() {
        return String::new();
    }
    // SAFETY: the plugin returns NUL-terminated strings that stay valid while it is loaded; the
    // text is copied at once.
    unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned()
}
