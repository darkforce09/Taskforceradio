//! `RunningPlugin`: everything `ts3plugin_init` starts, and the order `ts3plugin_shutdown` stops
//! it in.
//!
//! **Role:** start the plugin: the core with its resources and the connections already up, the
//! game link server with the bridge, the service thread, the push-to-talk release thread and the
//! update check; and stop it within TeamSpeak's patience: no more game messages, the threads
//! joined, the running game ended, the disabled status written, every mute lifted, the state
//! cleared.
//!
//! **Position:** owned by `TfarPlugin` behind its mutex, made in `init` and consumed in
//! `shutdown`, both on TeamSpeak's main thread.
//!
//! **Signals and state:** the thread handles and the server; the core is shared with every
//! thread through its `Arc`.
//!
//! **Invariants:** after `stop` the link server's threads, the service thread and the release
//! thread are joined, so no plugin thread still runs code of the library; the threads stop
//! before the end-of-game side effects run, so those never race a tick or a game message.

use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

use tfar_game_link_server::prelude::GameLinkServer;
use tfar_ts3_host::{Ts3, print_to_current_tab};
use tfar_web_requests::{UPDATE_AVAILABLE_MESSAGE, spawn_update_check};

use crate::game_messages::GameLinkBridge;
use crate::listen_port::server_config_from_environment;
use crate::plugin_core::PluginCore;
use crate::plugin_lifecycle::FLAVOUR;
use crate::plugin_startup::gather_resources;
use crate::service_thread::ServiceThread;
use crate::status_metadata::PLUGIN_VERSION;
use crate::team_speak::{TeamSpeak, Ts3TeamSpeak};

/// The environment variable that, set to `1`, skips the update check: the plugin test suites
/// set it so no test reaches the network.
pub const SKIP_UPDATE_CHECK_VARIABLE: &str = "TFAR_SKIP_UPDATE_CHECK";

/// The name of the push-to-talk release thread.
const RELEASE_THREAD_NAME: &str = "tfar-push-to-talk-release";

/// Why the plugin did not start.
#[derive(Debug, thiserror::Error)]
pub enum StartError {
    /// The game link could not start listening.
    #[error("the game link did not start: {0}")]
    GameLink(#[from] tfar_game_link_server::Error),
    /// A thread of the plugin could not start.
    #[error("the {thread} thread did not start: {source}")]
    Thread {
        /// The thread's name.
        thread: &'static str,
        /// The operating system's error.
        #[source]
        source: std::io::Error,
    },
}

/// What `init` started; see the module header.
#[derive(Debug)]
pub struct RunningPlugin {
    core: Arc<PluginCore>,
    server: GameLinkServer,
    service: ServiceThread,
    release_thread: JoinHandle<()>,
    update_check: Option<JoinHandle<()>>,
}

impl RunningPlugin {
    /// Starts the plugin over the installed function table `ts3`.
    ///
    /// # Errors
    ///
    /// [`StartError`] when the game link or a thread cannot start; whatever started is stopped
    /// again.
    pub fn start(ts3: &'static Ts3) -> Result<Self, StartError> {
        let team_speak: Arc<dyn TeamSpeak> = Arc::new(Ts3TeamSpeak::new(ts3));
        let resources = gather_resources(ts3, &*team_speak);
        let core = Arc::new(PluginCore::new(
            Arc::clone(&team_speak),
            resources,
            Instant::now(),
        ));
        for server in team_speak.connected_servers() {
            core.add_connection(server);
        }
        let config = server_config_from_environment(FLAVOUR);
        let server =
            GameLinkServer::start(config, Arc::new(GameLinkBridge::new(Arc::clone(&core))))?;
        tracing::info!(port = config.port, "started; the game link listens");
        let service = match ServiceThread::start(Arc::clone(&core)) {
            Ok(service) => service,
            Err(source) => {
                server.shutdown();
                return Err(StartError::Thread {
                    thread: "service",
                    source,
                });
            }
        };
        let release_core = Arc::clone(&core);
        let release_thread = match std::thread::Builder::new()
            .name(RELEASE_THREAD_NAME.to_owned())
            .spawn(move || {
                release_core
                    .push_to_talk()
                    .run_release_thread(release_core.team_speak());
            }) {
            Ok(thread) => thread,
            Err(source) => {
                server.shutdown();
                service.stop();
                return Err(StartError::Thread {
                    thread: RELEASE_THREAD_NAME,
                    source,
                });
            }
        };
        Ok(Self {
            core,
            server,
            service,
            release_thread,
            update_check: start_update_check(),
        })
    }

    /// The shared core.
    pub fn core(&self) -> &Arc<PluginCore> {
        &self.core
    }

    /// Stops everything `start` started; see the module header.
    pub fn stop(self) {
        let Self {
            core,
            server,
            service,
            release_thread,
            update_check,
        } = self;
        server.shutdown();
        service.stop();
        core.push_to_talk().stop();
        if release_thread.join().is_err() {
            tracing::warn!("the push-to-talk release thread ended abnormally");
        }
        let current = core.team_speak().current_server();
        if core.is_in_game() {
            core.end_game(current, false);
        }
        for connection in core.connections() {
            core.publish_disabled_metadata(connection.server);
            core.unmute_everyone(connection.server);
        }
        core.clear_connections();
        if let Some(update_check) = update_check {
            if update_check.is_finished() {
                let _ = update_check.join();
            } else {
                tracing::warn!(
                    "the update check is still waiting for the network; it is left to end alone"
                );
            }
        }
    }
}

/// Starts the update check unless [`SKIP_UPDATE_CHECK_VARIABLE`] is `1`; a newer published
/// version prints the original's notice in the chat tab.
fn start_update_check() -> Option<JoinHandle<()>> {
    if std::env::var(SKIP_UPDATE_CHECK_VARIABLE).as_deref() == Ok("1") {
        return None;
    }
    spawn_update_check(PLUGIN_VERSION, || {
        let _ = print_to_current_tab(UPDATE_AVAILABLE_MESSAGE);
    })
    .map_err(|error| tracing::warn!(%error, "the update check did not start"))
    .ok()
}
