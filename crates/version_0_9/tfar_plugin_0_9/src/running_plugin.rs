//! `RunningPlugin`: everything `ts3plugin_init` starts, and the order `ts3plugin_shutdown` stops
//! it in.
//!
//! **Role:** start the plugin: the core with its resources and the connections already up, the
//! game link server with the bridge, the service thread, the push-to-talk release thread and the
//! update check; and stop it within TeamSpeak's patience (`SHUTDOWN_BUDGET`, 2 s): no more game
//! messages, the web requests cancelled, the threads joined, the running game ended, the
//! disabled status written, every mute lifted, the state cleared, the web requests joined until
//! `WEB_REQUEST_JOIN_DEADLINE`.
//!
//! **Position:** owned by `TfarPlugin` behind its mutex, made in `init` and consumed in
//! `shutdown`, both on TeamSpeak's main thread.
//!
//! **Signals and state:** the thread handles and the server; the core is shared with every
//! thread through its `Arc` and keeps the web request handles.
//!
//! **Invariants:** after `stop` the link server's threads, the service thread and the release
//! thread are joined, and so is every web request that ended by the deadline; a request whose
//! network call is still in flight then (it cannot be interrupted) is cancelled, logged and ends
//! within its own timeout, 10 s for the update check and 5 s for a tracking request. The threads
//! stop before the end-of-game side effects run, so those never race a tick or a game message.

use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use tfar_game_link::server::prelude::GameLinkServer;
use tfar_ts3_host::{Ts3, print_to_current_tab};
use tfar_web_requests::{UPDATE_AVAILABLE_MESSAGE, spawn_cancellable_update_check};

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

/// How long TeamSpeak may wait in `ts3plugin_shutdown`.
pub const SHUTDOWN_BUDGET: Duration = Duration::from_secs(2);

/// How long after the start of the shutdown the web requests may still be joined; past it, a
/// request still in flight is logged and left to end within its own timeout, so the shutdown
/// stays within [`SHUTDOWN_BUDGET`].
pub const WEB_REQUEST_JOIN_DEADLINE: Duration = Duration::from_millis(1_500);

/// Why the plugin did not start.
#[derive(Debug, thiserror::Error)]
pub enum StartError {
    /// The game link could not start listening.
    #[error("the game link did not start: {0}")]
    GameLink(#[from] tfar_game_link::server::Error),
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

/// What `init` started; see the module header. The update check and the tracking requests are
/// kept in the core's web requests.
#[derive(Debug)]
pub struct RunningPlugin {
    core: Arc<PluginCore>,
    server: GameLinkServer,
    service: ServiceThread,
    release_thread: JoinHandle<()>,
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
        tracing::info!(
            port = config.port,
            debug_port = ?config.debug_port,
            "started; the game link listens"
        );
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
        start_update_check(&core);
        Ok(Self {
            core,
            server,
            service,
            release_thread,
        })
    }

    /// The shared core.
    pub fn core(&self) -> &Arc<PluginCore> {
        &self.core
    }

    /// Stops everything `start` started, within [`SHUTDOWN_BUDGET`]; see the module header.
    pub fn stop(self) {
        let started = Instant::now();
        let Self {
            core,
            server,
            service,
            release_thread,
        } = self;
        server.shutdown();
        core.web_requests().cancel_all();
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
        let web_request_deadline = started
            .checked_add(WEB_REQUEST_JOIN_DEADLINE)
            .unwrap_or(started);
        core.web_requests().cancel_and_join(web_request_deadline);
        tracing::debug!(elapsed = ?started.elapsed(), "the plugin's threads are stopped");
    }
}

/// Starts the update check unless [`SKIP_UPDATE_CHECK_VARIABLE`] is `1`, kept in the core's web
/// requests for shutdown to cancel and join; a newer published version prints the original's
/// notice in the chat tab.
fn start_update_check(core: &PluginCore) {
    if std::env::var(SKIP_UPDATE_CHECK_VARIABLE).as_deref() == Ok("1") {
        return;
    }
    match spawn_cancellable_update_check(PLUGIN_VERSION, || {
        let _ = print_to_current_tab(UPDATE_AVAILABLE_MESSAGE);
    }) {
        Ok(update_check) => core.web_requests().keep(update_check),
        Err(error) => tracing::warn!(%error, "the update check did not start"),
    }
}
