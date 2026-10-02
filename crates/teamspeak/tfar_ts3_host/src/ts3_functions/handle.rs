//! [`Ts3`]: the copied TeamSpeak function table and the plugin ID, installed once per load.
//!
//! **Role:** owns the `TS3Functions` table TeamSpeak passes to `ts3plugin_setFunctionPointers` and
//! turns TeamSpeak's error codes into [`Ts3Error`]s. The per-function methods live in the sibling
//! files of this folder, one group of the SDK each.
//!
//! **Position:** the base of `ts3_functions`; used by every method file, by `plugin_exports` (which
//! installs the table and registers the ID) and by `logging`.
//!
//! **Signals and state:** one process-wide `OnceLock<Ts3>`, written on the TeamSpeak main thread
//! by `ts3plugin_setFunctionPointers` and read from every thread afterwards; the plugin ID is a
//! `OnceLock` inside it, written by `ts3plugin_registerPluginID`.
//!
//! **Invariants:** the table is set once and never changes; calling a function pointer needs no
//! lock, so no TeamSpeak call ever runs under a lock of this crate.

use std::ffi::{CStr, CString, c_char, c_uint};
use std::sync::OnceLock;

use ts3plugin_sys::ts3functions::Ts3Functions;

use crate::Ts3Error;

/// TeamSpeak's `ERROR_ok`.
const ERROR_OK: c_uint = 0x0000;
/// TeamSpeak's `ERROR_ok_no_update`: success, nothing changed (a flush with no pending update).
const ERROR_OK_NO_UPDATE: c_uint = 0x0003;
/// The text used when TeamSpeak has no message for an error code.
const UNKNOWN_ERROR_TEXT: &str = "TeamSpeak gave no text for this error code";

/// The process-wide table, set by `ts3plugin_setFunctionPointers`.
static INSTALLED: OnceLock<Ts3> = OnceLock::new();

/// The TeamSpeak host: one safe method per SDK function TFAR calls.
///
/// A method returns `Result<_, Ts3Error>` when TeamSpeak can fail or the argument cannot be passed;
/// strings and lists TeamSpeak allocates are copied into Rust values and freed with TeamSpeak's
/// `freeMemory` before the method returns.
pub struct Ts3 {
    /// The table as TeamSpeak handed it over.
    pub(crate) functions: Ts3Functions,
    /// The ID TeamSpeak gives the plugin in `ts3plugin_registerPluginID`.
    plugin_id: OnceLock<CString>,
}

impl Ts3 {
    /// A handle over `functions`, without installing it process-wide.
    pub(crate) fn from_functions(functions: Ts3Functions) -> Self {
        Self {
            functions,
            plugin_id: OnceLock::new(),
        }
    }

    /// The handle `ts3plugin_setFunctionPointers` installed, if TeamSpeak has called it.
    pub fn installed() -> Option<&'static Self> {
        INSTALLED.get()
    }

    /// The installed handle, or [`Ts3Error::FunctionsNotInstalled`].
    ///
    /// # Errors
    ///
    /// [`Ts3Error::FunctionsNotInstalled`] before `ts3plugin_setFunctionPointers`.
    pub fn installed_or_error() -> Result<&'static Self, Ts3Error> {
        Self::installed().ok_or(Ts3Error::FunctionsNotInstalled)
    }

    /// Installs `functions` process-wide; a second call keeps the first table, which is the same
    /// table for the life of the loaded plugin.
    pub(crate) fn install(functions: Ts3Functions) -> &'static Self {
        INSTALLED.get_or_init(|| Self::from_functions(functions))
    }

    /// Records the plugin ID; the first ID TeamSpeak sends is kept.
    pub(crate) fn register_plugin_id(&self, plugin_id: &CStr) {
        let _ = self.plugin_id.set(plugin_id.to_owned());
    }

    /// The plugin ID TeamSpeak registered, if it has.
    pub fn plugin_id(&self) -> Option<&CStr> {
        self.plugin_id.get().map(CString::as_c_str)
    }

    /// The plugin ID, or [`Ts3Error::PluginIdNotRegistered`] naming `function`.
    pub(crate) fn plugin_id_for(&self, function: &'static str) -> Result<&CStr, Ts3Error> {
        self.plugin_id()
            .ok_or(Ts3Error::PluginIdNotRegistered { function })
    }

    /// `Ok` for TeamSpeak's success codes, else the [`Ts3Error::Call`] with TeamSpeak's text.
    pub(crate) fn check(&self, function: &'static str, code: c_uint) -> Result<(), Ts3Error> {
        if code == ERROR_OK || code == ERROR_OK_NO_UPDATE {
            return Ok(());
        }
        Err(Ts3Error::Call {
            function,
            code,
            message: self.error_message(code),
        })
    }

    /// `getErrorMessage`: TeamSpeak's text for `code`, or a fallback when it has none.
    pub fn error_message(&self, code: u32) -> String {
        let mut text: *mut c_char = std::ptr::null_mut();
        let answer = (self.functions.get_error_message)(code, &raw mut text);
        if answer != ERROR_OK || text.is_null() {
            return UNKNOWN_ERROR_TEXT.to_owned();
        }
        // SAFETY: `getErrorMessage` succeeded and left `text` pointing at a NUL-terminated string
        // TeamSpeak allocated; ownership passes to us and `take_string` frees it with `freeMemory`.
        unsafe { self.take_string(text) }
    }

    /// Converts `text` to a C string for `function`, or reports where its NUL byte is.
    pub(crate) fn c_string(function: &'static str, text: &str) -> Result<CString, Ts3Error> {
        CString::new(text).map_err(|error| Ts3Error::InteriorNul {
            function,
            position: error.nul_position(),
        })
    }
}
