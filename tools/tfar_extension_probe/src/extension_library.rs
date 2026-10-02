//! A loaded Arma extension library and its two exports, called the way Arma calls them.
//!
//! **Role:** loads the library with `libloading`, resolves `RVExtension` (required) and
//! `RVExtensionVersion` (optional, as for Arma), and makes each call with an output buffer of
//! [`OUTPUT_BUFFER_SIZE`] bytes, the size Arma passes.
//!
//! **Position:** called by the probe's `main`; the only code of the probe that touches raw
//! pointers.
//!
//! **Signals and state:** the loaded library, kept for as long as the resolved function pointers
//! are used; every call runs on the calling thread.
//!
//! **Invariants:** a call text holds no NUL byte, so the C string Arma would pass is the whole
//! text; the buffer handed over is always `OUTPUT_BUFFER_SIZE` bytes and zeroed before each call,
//! and the reply is read up to its first NUL and never past the buffer.

use std::ffi::{CString, c_char, c_int};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

#[cfg(test)]
#[path = "tests/extension_library.rs"]
mod tests;

/// The output buffer size Arma 3 passes to `RVExtension` and `RVExtensionVersion`.
pub(crate) const OUTPUT_BUFFER_SIZE: usize = 10240;

/// `void RVExtension(char* output, int outputSize, const char* function)`.
type ExtensionCall = unsafe extern "system" fn(*mut c_char, c_int, *const c_char);

/// `void RVExtensionVersion(char* output, int outputSize)`.
type ExtensionVersion = unsafe extern "system" fn(*mut c_char, c_int);

/// A loaded extension library with its resolved exports.
pub(crate) struct ExtensionLibrary {
    path: PathBuf,
    call: ExtensionCall,
    version: Option<ExtensionVersion>,
    /// Kept loaded while `call` and `version` point into it; dropped last.
    _library: libloading::Library,
}

impl ExtensionLibrary {
    /// Loads the library at `path` and resolves its exports.
    ///
    /// # Errors
    ///
    /// When the library does not load or does not export `RVExtension`.
    pub(crate) fn load(path: &Path) -> Result<Self> {
        // SAFETY: loading runs the library's initialisers; the probe loads only the extension the
        // developer names, which is exactly what Arma does with the same file.
        let library = unsafe { libloading::Library::new(path) }
            .with_context(|| format!("loading the extension library {}", path.display()))?;
        // SAFETY: `RVExtension` has this signature in the Arma extension ABI; the pointer is used
        // only while `library` stays loaded inside the returned value.
        let call = unsafe { library.get::<ExtensionCall>(b"RVExtension") }
            .map(|symbol| *symbol)
            .with_context(|| format!("{} exports no RVExtension", path.display()))?;
        // SAFETY: as above, for `RVExtensionVersion`, which Arma treats as optional.
        let version = unsafe { library.get::<ExtensionVersion>(b"RVExtensionVersion") }
            .ok()
            .map(|symbol| *symbol);
        Ok(Self {
            path: path.to_owned(),
            call,
            version,
            _library: library,
        })
    }

    /// The path the library was loaded from.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// What `RVExtensionVersion` writes, or `None` when the library does not export it.
    pub(crate) fn version(&self) -> Option<String> {
        let version = self.version?;
        let mut buffer = vec![0_u8; OUTPUT_BUFFER_SIZE];
        let size = buffer_size_argument();
        // SAFETY: `buffer` holds `size` writable bytes for the whole call, and the export has the
        // ABI's signature (checked by name when it was resolved).
        unsafe { version(buffer.as_mut_ptr().cast::<c_char>(), size) };
        Some(reply_from_buffer(&buffer))
    }

    /// Calls `RVExtension` with `text` and returns the reply.
    ///
    /// # Errors
    ///
    /// When `text` holds a NUL byte, which no Arma call can carry.
    pub(crate) fn call(&self, text: &str) -> Result<String> {
        let Ok(function) = CString::new(text) else {
            bail!("the call {text:?} holds a NUL byte, which no Arma call can carry");
        };
        let mut buffer = vec![0_u8; OUTPUT_BUFFER_SIZE];
        let size = buffer_size_argument();
        // SAFETY: `buffer` holds `size` writable bytes and `function` is a NUL-terminated string,
        // both alive for the whole call, and the export has the ABI's signature.
        unsafe {
            (self.call)(
                buffer.as_mut_ptr().cast::<c_char>(),
                size,
                function.as_ptr(),
            );
        }
        Ok(reply_from_buffer(&buffer))
    }
}

/// [`OUTPUT_BUFFER_SIZE`] as the `int` the exports take.
fn buffer_size_argument() -> c_int {
    c_int::try_from(OUTPUT_BUFFER_SIZE).unwrap_or(c_int::MAX)
}

/// The reply in `buffer`: the bytes before the first NUL (all of them when there is none),
/// decoded lossily.
pub(crate) fn reply_from_buffer(buffer: &[u8]) -> String {
    let end = buffer
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(buffer.len());
    String::from_utf8_lossy(buffer.get(..end).unwrap_or_default()).into_owned()
}
