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
//! by `ts3plugin_setFunctionPointers` and read from every thread afterwards; the plugin ID is an
//! `ArcSwapOption` inside it, replaced by every `ts3plugin_registerPluginID`. TeamSpeak's
//! "Reload" keeps the library loaded (so the table survives) but registers a new ID, which
//! replaces the old one.
//!
//! **Invariants:** the table is set once and never changes; the plugin ID is the last one
//! TeamSpeak registered; calling a function pointer needs no lock, so no TeamSpeak call ever runs
//! under a lock of this crate.

use std::ffi::{CStr, CString, c_char, c_uint};
use std::sync::{Arc, OnceLock};

use arc_swap::ArcSwapOption;
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
    /// The ID TeamSpeak last gave the plugin in `ts3plugin_registerPluginID`.
    plugin_id: ArcSwapOption<CString>,
}

impl Ts3 {
    /// A handle over `functions`, without installing it process-wide.
    pub(crate) fn from_functions(functions: Ts3Functions) -> Self {
        Self {
            functions,
            plugin_id: ArcSwapOption::const_empty(),
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

    /// Records the plugin ID; every ID TeamSpeak sends replaces the one before, so after a
    /// "Reload" the calls carry the new ID.
    pub(crate) fn register_plugin_id(&self, plugin_id: &CStr) {
        self.plugin_id.store(Some(Arc::new(plugin_id.to_owned())));
    }

    /// The plugin ID TeamSpeak registered last, if it has registered one: a snapshot that stays
    /// valid however often TeamSpeak registers a new ID afterwards.
    pub fn plugin_id(&self) -> Option<Arc<CString>> {
        self.plugin_id.load_full()
    }

    /// The plugin ID, or [`Ts3Error::PluginIdNotRegistered`] naming `function`.
    pub(crate) fn plugin_id_for(&self, function: &'static str) -> Result<Arc<CString>, Ts3Error> {
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
