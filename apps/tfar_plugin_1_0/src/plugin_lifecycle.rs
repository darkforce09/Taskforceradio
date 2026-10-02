//! The plugin TeamSpeak loads: its metadata, and what `init` starts and `shutdown` stops.
//!
//! **Role:** [`TfarPlugin`] implements [`Ts3Plugin`] with the original 1.0 plugin's metadata;
//! `init` installs the TeamSpeak log forwarder and starts the game link server with the
//! [`StubGameLinkHandler`], `shutdown` stops both.
//!
//! **Position:** the `static` the crate root hands to `ts3_plugin!`; starts
//! [`GameLinkServer`] and the logging bridge of `tfar_ts3_host`.
//!
//! **Signals and state:** the running server and the logging guard behind one mutex, taken only
//! on TeamSpeak's main thread in `init` and `shutdown`.
//!
//! **Invariants:** after `shutdown` no thread the plugin started is running: the server's
//! threads are joined before the logging guard drops, so the server's last lines still reach
//! TeamSpeak's log; a second `init` stops what the first one started before starting again.

use std::ffi::CStr;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use tfar_game_link_server::prelude::{Flavour, GameLinkServer};
use tfar_ts3_host::{
    InitStatus, LoggingGuard, Ts3, Ts3Plugin, install_logging, print_to_current_tab,
};

use crate::game_link_stub::StubGameLinkHandler;
use crate::listen_port::server_config_from_environment;

#[cfg(test)]
#[path = "tests/plugin_lifecycle.rs"]
mod tests;

/// The mod version this plugin serves.
pub const FLAVOUR: Flavour = Flavour::V1_0;

/// The channel name the plugin's lines carry in TeamSpeak's log.
pub const LOG_CHANNEL: &str = "tfar_plugin_1_0";

/// The 1.0 TeamSpeak plugin; see the module header.
pub struct TfarPlugin {
    running: Mutex<Option<RunningPlugin>>,
}

/// What `init` started.
struct RunningPlugin {
    server: GameLinkServer,
    /// `None` when the log forwarder could not be installed; the plugin runs without it.
    logging: Option<LoggingGuard>,
}

impl TfarPlugin {
    /// A plugin that has started nothing.
    pub const fn new() -> Self {
        Self {
            running: Mutex::new(None),
        }
    }

    /// Whether `init` started the plugin and `shutdown` has not stopped it.
    pub fn is_running(&self) -> bool {
        self.lock_running().is_some()
    }

    /// Stops the server, joining its threads, then the log forwarder.
    fn stop(&self) {
        let running = self.lock_running().take();
        if let Some(RunningPlugin { server, logging }) = running {
            server.shutdown();
            tracing::info!("stopped");
            drop(logging);
        }
    }

    fn lock_running(&self) -> MutexGuard<'_, Option<RunningPlugin>> {
        // The guarded value is replaced whole, so a poisoned lock is still consistent (and with
        // `panic = "abort"` it never poisons).
        self.running.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for TfarPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for TfarPlugin {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("TfarPlugin")
            .field("is_running", &self.is_running())
            .finish_non_exhaustive()
    }
}

impl Ts3Plugin for TfarPlugin {
    const NAME: &'static CStr = c"Task Force Arrowhead Radio";
    const VERSION: &'static CStr = c"1.0.0.1";
    const AUTHOR: &'static CStr = c"[TF]Nkey and Dedmen";
    const DESCRIPTION: &'static CStr = c"Radio Addon for Arma 3";
    const INFO_TITLE: &'static CStr = c"Task Force Radio Status";
    const COMMAND_KEYWORD: &'static CStr = c"tfar";

    fn init(&self, _ts3: &'static Ts3) -> InitStatus {
        self.stop();
        let logging = match install_logging(LOG_CHANNEL) {
            Ok(guard) => Some(guard),
            Err(error) => {
                // Without the forwarder the chat tab is the only place left to say so.
                let _ = print_to_current_tab(&format!(
                    "{LOG_CHANNEL}: the TeamSpeak log forwarder did not start: {error}"
                ));
                None
            }
        };
        let config = server_config_from_environment(FLAVOUR);
        match GameLinkServer::start(config, Arc::new(StubGameLinkHandler)) {
            Ok(server) => {
                tracing::info!(port = config.port, "started; the game link listens");
                *self.lock_running() = Some(RunningPlugin { server, logging });
                InitStatus::Success
            }
            Err(error) => {
                tracing::error!(%error, "the game link did not start");
                drop(logging);
                InitStatus::Failure
            }
        }
    }

    fn shutdown(&self) {
        self.stop();
    }
}
