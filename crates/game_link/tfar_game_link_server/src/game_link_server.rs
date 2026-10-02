//! The game link server handle the plugin keeps from load to unload.
//!
//! **Role:** starts the accept thread and the async worker, exposes needcfg, the link state and
//! the liveness values, and stops and joins every thread on shutdown.
//!
//! **Position:** created by the plugin app in `ts3plugin_init`, shut down in `ts3plugin_shutdown`;
//! the threads it starts run the accept loop, the session loop and the async worker.
//!
//! **Signals and state:** the shared [`LinkState`] and the join handles of the accept thread and
//! the worker; every method takes `&self` and may be called from any thread.
//!
//! **Invariants:** a needcfg change reaches the current session as one `STATUS` at once; shutdown
//! returns within one poll period plus the longest running handler call, well under 1 s; dropping
//! the server shuts it down too.

use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crossbeam_channel::Receiver;

use crate::accept_loop::{bind_listener, run_accept_loop};
use crate::async_worker::{AsyncMessage, run_async_worker};
use crate::error::{Error, Result};
use crate::game_link_handler::GameLinkHandler;
use crate::link_state::LinkState;
use crate::server_config::ServerConfig;

/// The name of the accept thread.
const ACCEPT_THREAD_NAME: &str = "tfar-game-link-accept";
/// The name of the async worker thread.
const ASYNC_THREAD_NAME: &str = "tfar-game-link-async";

/// A running game link server; see the module header.
pub struct GameLinkServer {
    /// The state its threads share.
    state: Arc<LinkState>,
    /// The accept thread, which joins the session threads before it ends.
    accept_thread: Option<JoinHandle<()>>,
    /// The async worker.
    async_thread: Option<JoinHandle<()>>,
}

impl GameLinkServer {
    /// Starts the server: one bind attempt on `127.0.0.1:<config.port>` now, then the accept thread
    /// and the async worker. A busy port is logged and retried every
    /// [`ServerConfig::bind_retry_interval`]; the server stays dormant meanwhile.
    ///
    /// # Errors
    ///
    /// [`Error::ThreadSpawnFailed`] when the system refuses a thread; nothing keeps running then.
    pub fn start(config: ServerConfig, handler: Arc<dyn GameLinkHandler>) -> Result<Self> {
        let (sender, receiver) = crossbeam_channel::unbounded::<AsyncMessage>();
        let state = Arc::new(LinkState::new(config, handler, sender));
        let listener = bind_listener(&state, true);
        let mut server = Self {
            state,
            accept_thread: None,
            async_thread: None,
        };
        server.async_thread = Some(spawn_async_worker(&server.state, receiver)?);
        match spawn_accept_thread(&server.state, listener) {
            Ok(handle) => server.accept_thread = Some(handle),
            Err(error) => {
                server.stop_and_join();
                return Err(error);
            }
        }
        Ok(server)
    }

    /// The address the listener is bound to; `None` while the port is still busy.
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.state.local_addr()
    }

    /// Sets needcfg; when the value changes, the current session gets one `STATUS` at once (the
    /// 0.9 server always sends false). May block for up to the write timeout on a stalled socket.
    pub fn set_needcfg(&self, needcfg: bool) {
        let previous = self.state.swap_needcfg(needcfg);
        if previous != needcfg
            && let Some(session) = self.state.current_session()
        {
            // A failed push closes the session; its thread reports the close.
            session.send_status(&self.state);
        }
    }

    /// The needcfg value set last (or [`ServerConfig::initial_needcfg`]).
    pub fn needcfg(&self) -> bool {
        self.state.needcfg()
    }

    /// True while a session that passed the handshake is open: the link is connected.
    pub fn is_session_up(&self) -> bool {
        self.state.is_session_up()
    }

    /// When the last game message arrived: a `SYNC_REQ` or an `ASYNC` other than `MISSIONEND`.
    /// `HELLO` and `FOCUS` never count; a `MISSIONEND` resets it to `None`. A closed session leaves
    /// it as it was, so the plugin's game timeout runs out on its own.
    pub fn last_game_message(&self) -> Option<Instant> {
        self.state.last_game_message()
    }

    /// Stops every thread and joins it: the accept thread, each session thread (whose session is
    /// reported closed) and the async worker, which leaves unrun messages behind. Returns within
    /// one poll period plus the longest handler call still running.
    pub fn shutdown(mut self) {
        self.stop_and_join();
    }

    /// The body of [`shutdown`](Self::shutdown) and of `Drop`.
    fn stop_and_join(&mut self) {
        self.state.begin_stopping();
        for (name, handle) in [
            (ACCEPT_THREAD_NAME, self.accept_thread.take()),
            (ASYNC_THREAD_NAME, self.async_thread.take()),
        ] {
            if let Some(handle) = handle
                && handle.join().is_err()
            {
                tracing::error!(thread = name, "game link: a thread panicked");
            }
        }
    }
}

impl Drop for GameLinkServer {
    fn drop(&mut self) {
        self.stop_and_join();
    }
}

impl core::fmt::Debug for GameLinkServer {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("GameLinkServer")
            .field("config", &self.state.config)
            .field("local_addr", &self.local_addr())
            .field("is_session_up", &self.is_session_up())
            .finish_non_exhaustive()
    }
}

/// Starts the async worker over `receiver`.
fn spawn_async_worker(
    state: &Arc<LinkState>,
    receiver: Receiver<AsyncMessage>,
) -> Result<JoinHandle<()>> {
    let worker_state = Arc::clone(state);
    thread::Builder::new()
        .name(ASYNC_THREAD_NAME.to_owned())
        .spawn(move || run_async_worker(&worker_state, &receiver))
        .map_err(|source| Error::ThreadSpawnFailed {
            thread_name: ASYNC_THREAD_NAME,
            source,
        })
}

/// Starts the accept thread with the listener of the first bind attempt.
fn spawn_accept_thread(
    state: &Arc<LinkState>,
    listener: Option<TcpListener>,
) -> Result<JoinHandle<()>> {
    let accept_state = Arc::clone(state);
    thread::Builder::new()
        .name(ACCEPT_THREAD_NAME.to_owned())
        .spawn(move || run_accept_loop(&accept_state, listener))
        .map_err(|source| Error::ThreadSpawnFailed {
            thread_name: ACCEPT_THREAD_NAME,
            source,
        })
}
