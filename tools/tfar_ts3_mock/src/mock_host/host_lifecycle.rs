//! [`MockHost`]: a started plugin inside the fake TeamSpeak, from load to shutdown.
//!
//! **Role:** starts a [`PluginLibrary`] the way TeamSpeak does (`setFunctionPointers`, then
//! `registerPluginID("mock_plugin_id")`, then `init`), owns the host session while the plugin
//! runs, answers the call log and the leak report, drives `infoData` and `processCommand`, and
//! shuts the plugin down exactly once.
//!
//! **Position:** the entry point of the crate; the event and audio drivers are `impl MockHost`
//! blocks in the sibling files. Used by the plugin apps' integration tests and the binary.
//!
//! **Signals and state:** the loaded library and the [`HostSession`] that owns the process-global
//! fake state; every plugin call runs on the caller's thread with no lock held.
//!
//! **Invariants:** at most one `MockHost` is alive per process (others wait in
//! [`MockHost::start`]); `ts3plugin_shutdown` runs once, before the library unloads; an
//! `infoData` block is handed back to the plugin's `ts3plugin_freeMemory`.

use std::ffi::{CStr, CString, c_char, c_int};
use std::path::{Path, PathBuf};

use tfar_ts3_host::ServerConnectionHandlerId;
use ts3plugin_sys::plugin_definitions::ItemType;

use crate::fake_functions::fake_table::fake_function_table;
use crate::fake_state::host_session::{HostSession, with_fake_teamspeak};
use crate::plugin_library::loaded_plugin::copied_text;
use crate::{
    Error, FakeTeamSpeak, HostCall, LeakReport, PluginExports, PluginLibrary, PluginMetadata,
    Result,
};

/// The plugin ID the mock registers, as TeamSpeak registers one per loaded plugin.
pub const MOCK_PLUGIN_ID: &str = "mock_plugin_id";

/// [`MOCK_PLUGIN_ID`] as the C string `registerPluginID` passes.
const MOCK_PLUGIN_ID_C_STRING: &CStr = c"mock_plugin_id";

#[cfg(test)]
#[path = "tests/host_lifecycle.rs"]
mod tests;

/// The plugin API the mock hosts: TeamSpeak 3.6's API 26.
pub const HOSTED_API_VERSION: i32 = 26;

/// Folders a host uses instead of the temporary ones it would create.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostOptions {
    /// What `getPluginPath` answers (a folder the test fills with radio sounds, say).
    pub plugin_path: Option<PathBuf>,
    /// What `getConfigPath` answers (a folder holding a test `settings.db`, say).
    pub config_path: Option<PathBuf>,
}

/// The item kinds of `ts3plugin_infoData`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoKind {
    /// `PLUGIN_SERVER`.
    Server,
    /// `PLUGIN_CHANNEL`.
    Channel,
    /// `PLUGIN_CLIENT`.
    Client,
}

/// The answer of `ts3plugin_processCommand`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandOutcome {
    /// `0`: the plugin handled the command.
    Handled,
    /// `1`: the plugin did not handle it.
    NotHandled,
    /// Any other answer.
    Other(i32),
}

/// A plugin started inside the fake TeamSpeak.
#[derive(Debug)]
pub struct MockHost {
    /// Whether `ts3plugin_shutdown` has run.
    is_shut_down: bool,
    /// The started plugin; dropped (unloaded) before the session ends.
    plugin: PluginLibrary,
    /// The exclusive use of the fake state.
    _session: HostSession,
}

impl MockHost {
    /// Loads the plugin at `path` and starts it with temporary plugin and config folders.
    ///
    /// # Errors
    ///
    /// The errors of [`PluginLibrary::open`] and [`MockHost::start`].
    pub fn load(path: &Path) -> Result<Self> {
        Self::load_with_options(path, &HostOptions::default())
    }

    /// Loads the plugin at `path` and starts it with `options`.
    ///
    /// # Errors
    ///
    /// The errors of [`PluginLibrary::open`] and [`MockHost::start`].
    pub fn load_with_options(path: &Path, options: &HostOptions) -> Result<Self> {
        Self::start(PluginLibrary::open(path)?, options)
    }

    /// Starts an opened plugin: waits for any other host to end, resets the fake state, then
    /// calls `setFunctionPointers`, `registerPluginID` and `init` in TeamSpeak's order.
    ///
    /// # Errors
    ///
    /// [`Error::ApiVersionMismatch`] when the plugin is not built for API 26;
    /// [`Error::TemporaryFolder`] when a default folder cannot be made; [`Error::InitFailed`]
    /// when `init` answers non-zero.
    pub fn start(plugin: PluginLibrary, options: &HostOptions) -> Result<Self> {
        let found = plugin.metadata().api_version;
        if found != HOSTED_API_VERSION {
            return Err(Error::ApiVersionMismatch {
                expected: HOSTED_API_VERSION,
                found,
            });
        }
        let session = HostSession::begin(options.plugin_path.clone(), options.config_path.clone())?;
        let exports = *plugin.exports();
        // SAFETY: the table is fully initialised with `extern "C"` fakes that live for the whole
        // process; the plugin copies it.
        unsafe { (exports.set_function_pointers)(fake_function_table()) };
        if let Some(register_plugin_id) = exports.register_plugin_id {
            let plugin_id = MOCK_PLUGIN_ID_C_STRING;
            // SAFETY: the ID is NUL-terminated and outlives the call; the plugin copies it.
            unsafe { register_plugin_id(plugin_id.as_ptr()) };
        }
        // SAFETY: `init` takes no argument; the table and ID it may use are installed.
        let code = unsafe { (exports.init)() };
        if code != 0 {
            return Err(Error::InitFailed { code });
        }
        Ok(Self {
            is_shut_down: false,
            plugin,
            _session: session,
        })
    }

    /// The plugin's metadata.
    pub fn metadata(&self) -> &PluginMetadata {
        self.plugin.metadata()
    }

    /// The plugin's exports, for events no driver covers; valid while `self` lives.
    pub fn exports(&self) -> &PluginExports {
        self.plugin.exports()
    }

    /// The loaded plugin library.
    pub fn plugin(&self) -> &PluginLibrary {
        &self.plugin
    }

    /// Every call the plugin made so far, in order.
    pub fn calls(&self) -> Vec<HostCall> {
        with_fake_teamspeak(|fake| fake.calls.clone())
    }

    /// The calls made since the last take, and forgets them.
    pub fn take_calls(&self) -> Vec<HostCall> {
        with_fake_teamspeak(|fake| std::mem::take(&mut fake.calls))
    }

    /// The blocks still handed out and the bad frees seen so far.
    pub fn leak_report(&self) -> LeakReport {
        with_fake_teamspeak(|fake| fake.ledger.report())
    }

    /// Runs `change` on the fake state, to read it or set it up. `change` must not call the plugin
    /// or this host (whose methods use the same state): a nested use panics.
    pub fn with_fake_teamspeak<Answer>(
        &self,
        change: impl FnOnce(&mut FakeTeamSpeak) -> Answer,
    ) -> Answer {
        with_fake_teamspeak(change)
    }

    /// The folder `getPluginPath` answers with.
    pub fn plugin_path(&self) -> PathBuf {
        with_fake_teamspeak(|fake| fake.plugin_path.clone())
    }

    /// The folder `getConfigPath` answers with.
    pub fn config_path(&self) -> PathBuf {
        with_fake_teamspeak(|fake| fake.config_path.clone())
    }

    /// The tab in front; `0` before a server connects.
    pub fn current_server(&self) -> ServerConnectionHandlerId {
        with_fake_teamspeak(|fake| fake.current_server)
    }

    /// The text the plugin shows in TeamSpeak's info panel for item `id` of `kind` on the
    /// current server; `None` when it shows nothing.
    ///
    /// # Errors
    ///
    /// [`Error::MissingExport`] when the plugin has no `ts3plugin_infoData`.
    pub fn info_data(&self, id: u64, kind: InfoKind) -> Result<Option<String>> {
        let exports = self.exports();
        let info_data = exports.info_data.ok_or(Error::MissingExport {
            symbol: "ts3plugin_infoData",
        })?;
        let item_type = match kind {
            InfoKind::Server => ItemType::Server,
            InfoKind::Channel => ItemType::Channel,
            InfoKind::Client => ItemType::Client,
        } as c_int;
        let mut data: *mut c_char = std::ptr::null_mut();
        // SAFETY: `data` is a writable `char*` slot for the call, as TeamSpeak passes.
        unsafe { info_data(self.current_server().0, id, item_type, &raw mut data) };
        if data.is_null() {
            return Ok(None);
        }
        let text = copied_text(data);
        match exports.free_memory {
            // SAFETY: `data` is the block the plugin just handed over, given back once.
            Some(free_memory) => unsafe { free_memory(data.cast()) },
            None => with_fake_teamspeak(|fake| fake.ledger.note_unfreed_plugin_block()),
        }
        Ok(Some(text))
    }

    /// Types `text` after the plugin's command keyword in the chat (`/tfar <text>`).
    ///
    /// # Errors
    ///
    /// [`Error::MissingExport`] without `ts3plugin_processCommand`; [`Error::InteriorNul`] when
    /// `text` holds a NUL byte.
    pub fn process_command(&self, text: &str) -> Result<CommandOutcome> {
        let process_command = self.exports().process_command.ok_or(Error::MissingExport {
            symbol: "ts3plugin_processCommand",
        })?;
        let command = CString::new(text).map_err(|_| Error::InteriorNul {
            argument: "command",
        })?;
        // SAFETY: the command is NUL-terminated and outlives the call.
        let answer = unsafe { process_command(self.current_server().0, command.as_ptr()) };
        Ok(match answer {
            0 => CommandOutcome::Handled,
            1 => CommandOutcome::NotHandled,
            other => CommandOutcome::Other(other),
        })
    }

    /// Calls `ts3plugin_shutdown`, once; later calls and the drop do nothing more.
    pub fn shutdown(&mut self) {
        if self.is_shut_down {
            return;
        }
        self.is_shut_down = true;
        // SAFETY: `shutdown` takes no argument; the plugin stops its threads before returning.
        unsafe { (self.exports().shutdown)() };
    }
}

impl Drop for MockHost {
    fn drop(&mut self) {
        self.shutdown();
    }
}
