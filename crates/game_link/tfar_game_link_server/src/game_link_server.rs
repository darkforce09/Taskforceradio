//! The game link server handle the plugin keeps from load to unload.
//!
//! **Role:** starts one accept thread per listen port and the async worker, exposes needcfg, the
//! link state, the bound addresses and the liveness values, lets the plugin queue its own async
//! messages, and stops and joins every thread on shutdown.
//!
//! **Position:** created by the plugin app in `ts3plugin_init`, shut down in `ts3plugin_shutdown`;
//! the threads it starts run the accept loops, the session loop and the async worker.
//!
//! **Signals and state:** the shared [`LinkState`] and the join handles of the accept threads and
//! the worker; every method takes `&self` and may be called from any thread.
//!
//! **Invariants:** a needcfg change reaches the current session as one `STATUS` at once; a
//! [`queue_async`](GameLinkServer::queue_async) message joins the same FIFO queue as the `ASYNC`
//! frames; shutdown returns within one poll period plus the longest running handler call, well
//! under 1 s; dropping the server shuts it down too.

use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crossbeam_channel::Receiver;

use crate::accept_loop::{ListenPort, bind_listener, run_accept_loop};
use crate::async_worker::{AsyncMessage, run_async_worker};
use crate::error::{Error, Result};
use crate::game_link_handler::GameLinkHandler;
use crate::link_state::LinkState;
use crate::server_config::ServerConfig;

/// The name of the accept thread of the normal port.
const ACCEPT_THREAD_NAME: &str = "tfar-game-link-accept";
/// The name of the accept thread of the debug port.
const DEBUG_ACCEPT_THREAD_NAME: &str = "tfar-game-link-accept-debug";
/// The name of the async worker thread.
const ASYNC_THREAD_NAME: &str = "tfar-game-link-async";

/// A running game link server; see the module header.
pub struct GameLinkServer {
    /// The state its threads share.
    state: Arc<LinkState>,
    /// The accept threads, one per listen port; each joins its session threads before it ends.
    accept_threads: Vec<(&'static str, JoinHandle<()>)>,
    /// The async worker.
    async_thread: Option<JoinHandle<()>>,
}

impl GameLinkServer {
    /// Starts the server: one bind attempt now on `127.0.0.1` and each of
    /// [`ServerConfig::listen_ports`], then one accept thread per port and the async worker. A
    /// busy port is logged and retried every [`ServerConfig::bind_retry_interval`], independently
    /// of the other port; it stays dormant meanwhile.
    ///
    /// # Errors
    ///
    /// [`Error::ThreadSpawnFailed`] when the system refuses a thread; nothing keeps running then.
    pub fn start(config: ServerConfig, handler: Arc<dyn GameLinkHandler>) -> Result<Self> {
        let (sender, receiver) = crossbeam_channel::unbounded::<AsyncMessage>();
        let state = Arc::new(LinkState::new(config, handler, sender));
        let mut server = Self {
            state,
            accept_threads: Vec::new(),
            async_thread: None,
        };
        server.async_thread = Some(spawn_async_worker(&server.state, receiver)?);
        for (slot, port) in config.listen_ports().enumerate() {
            let listen_port = ListenPort { slot, port };
            let listener = bind_listener(&server.state, listen_port, true);
            let name = if slot == 0 {
                ACCEPT_THREAD_NAME
            } else {
                DEBUG_ACCEPT_THREAD_NAME
            };
            match spawn_accept_thread(&server.state, name, listen_port, listener) {
                Ok(handle) => server.accept_threads.push((name, handle)),
                Err(error) => {
                    server.stop_and_join();
                    return Err(error);
                }
            }
        }
        Ok(server)
    }

    /// The address the normal port ([`ServerConfig::port`]) is bound to; `None` while that port is
    /// still busy.
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.state.bound_address(0)
    }

    /// The addresses of every listen port bound so far: the normal port first, then the debug
    /// port. A port that is still busy is missing until its bind retry succeeds.
    pub fn local_addrs(&self) -> Vec<SocketAddr> {
        self.state.bound_addresses()
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

    /// Queues `text` (an async call without its `~`) for
    /// [`GameLinkHandler::async_message`] exactly as if the extension had sent it as an `ASYNC`
    /// frame now: it joins the same first-in, first-out queue behind every message already
    /// queued, it counts as a game message, and a `MISSIONEND` drops the messages queued before
    /// it. A sync `POS` answer uses it to schedule its async apply. Never blocks; callable from
    /// any thread, a handler callback included.
    ///
    /// Returns false, dropping `text`, once the server has stopped.
    pub fn queue_async(&self, text: &[u8]) -> bool {
        if self.state.is_stopping() {
            return false;
        }
        let queued = self.state.queue_async_text(text.to_vec());
        if !queued {
            tracing::debug!("game link: async worker stopped; queued message dropped");
        }
        queued
    }

    /// True while a session that passed the handshake is open: the link is connected.
    pub fn is_session_up(&self) -> bool {
        self.state.is_session_up()
    }

    /// When the last game message arrived: a `SYNC_REQ`, an `ASYNC` other than `MISSIONEND`, or
    /// a [`queue_async`](Self::queue_async) message other than `MISSIONEND`. `HELLO` and `FOCUS`
    /// never count; a `MISSIONEND` resets it to `None`. A closed session leaves it as it was, so
    /// the plugin's game timeout runs out on its own.
    pub fn last_game_message(&self) -> Option<Instant> {
        self.state.last_game_message()
    }

    /// Stops every thread and joins it: the accept threads, each session thread (whose session is
    /// reported closed) and the async worker, which leaves unrun messages behind. Returns within
    /// one poll period plus the longest handler call still running.
    pub fn shutdown(mut self) {
        self.stop_and_join();
    }

    /// The body of [`shutdown`](Self::shutdown) and of `Drop`.
    fn stop_and_join(&mut self) {
        self.state.begin_stopping();
        let accept_threads = self.accept_threads.drain(..);
        let async_thread = self
            .async_thread
            .take()
            .map(|handle| (ASYNC_THREAD_NAME, handle));
        for (name, handle) in accept_threads.chain(async_thread) {
            if handle.join().is_err() {
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
            .field("local_addrs", &self.local_addrs())
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

/// Starts the accept thread `name` of `listen_port` with the listener of its first bind attempt.
fn spawn_accept_thread(
    state: &Arc<LinkState>,
    name: &'static str,
    listen_port: ListenPort,
    listener: Option<TcpListener>,
) -> Result<JoinHandle<()>> {
    let accept_state = Arc::clone(state);
    thread::Builder::new()
        .name(name.to_owned())
        .spawn(move || run_accept_loop(&accept_state, listen_port, listener))
        .map_err(|source| Error::ThreadSpawnFailed {
            thread_name: name,
            source,
        })
}
