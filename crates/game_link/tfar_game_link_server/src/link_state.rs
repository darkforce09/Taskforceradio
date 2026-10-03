//! The state every thread of one game link server shares.
//!
//! **Role:** holds the configuration, the handler, the stop flag, needcfg, the liveness values,
//! the current session and the async queue's sending side, and orders the session callbacks.
//!
//! **Position:** created by [`GameLinkServer::start`](crate::GameLinkServer::start), shared by
//! `Arc` with the accept threads, the session threads and the async worker.
//!
//! **Signals and state:** atomics for the flags, the mission generation and the connection counter;
//! `parking_lot` locks, each held only for a copy or a swap, for the current session, the last game
//! message time and the bound address of each listen port; `session_order`, held across
//! `session_opened` and `session_closed` only.
//!
//! **Invariants:** at most one session is current; `session_closed` follows its `session_opened`
//! and comes before the next session's; the 0.9 server always reports needcfg false; no lock but
//! `session_order` is held across a handler call.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use crossbeam_channel::Sender;
use parking_lot::Mutex;
use tfar_game_link_protocol::Flavour;

use crate::async_worker::{AsyncMessage, is_mission_end};
use crate::game_link_handler::GameLinkHandler;
use crate::server_config::ServerConfig;
use crate::session_link::{SessionLink, SessionWriter};

/// The shared state of one server; see the module header.
pub(crate) struct LinkState {
    /// The settings the server started with.
    pub(crate) config: ServerConfig,
    /// The plugin's callbacks.
    pub(crate) handler: Arc<dyn GameLinkHandler>,
    /// Set once by shutdown; every thread checks it at least once per poll period.
    stopping: AtomicBool,
    /// The needcfg value the plugin set last.
    needcfg: AtomicBool,
    /// True between a session's `session_opened` and its `session_closed`.
    session_up: AtomicBool,
    /// Bumped by every `MISSIONEND`; the worker drops messages queued under an older value.
    mission_generation: AtomicU64,
    /// The session that answers `set_needcfg` pushes, if one is up.
    current_session: Mutex<Option<Arc<SessionLink>>>,
    /// When the last `SYNC_REQ` or non-`MISSIONEND` `ASYNC` arrived; `MISSIONEND` clears it.
    last_game_message: Mutex<Option<Instant>>,
    /// The address each listen port is bound to, in [`ServerConfig::listen_ports`] order; `None`
    /// until that port's bind succeeds.
    bound_addresses: Mutex<Vec<Option<SocketAddr>>>,
    /// The number the next accepted connection gets, shared by every accept thread.
    next_connection_id: AtomicU64,
    /// Orders the session callbacks and the session swaps.
    session_order: Mutex<()>,
    /// The sending side of the async worker's FIFO queue.
    async_queue: Sender<AsyncMessage>,
}

impl LinkState {
    /// The state of a server that is about to start its threads.
    pub(crate) fn new(
        config: ServerConfig,
        handler: Arc<dyn GameLinkHandler>,
        async_queue: Sender<AsyncMessage>,
    ) -> Self {
        Self {
            config,
            handler,
            stopping: AtomicBool::new(false),
            needcfg: AtomicBool::new(config.initial_needcfg),
            session_up: AtomicBool::new(false),
            mission_generation: AtomicU64::new(0),
            current_session: Mutex::new(None),
            last_game_message: Mutex::new(None),
            bound_addresses: Mutex::new(config.listen_ports().map(|_| None).collect()),
            next_connection_id: AtomicU64::new(1),
            session_order: Mutex::new(()),
            async_queue,
        }
    }

    /// True once shutdown has begun.
    pub(crate) fn is_stopping(&self) -> bool {
        self.stopping.load(Ordering::SeqCst)
    }

    /// Tells every thread to stop and closes the current session's socket, so a blocked write
    /// returns at once.
    pub(crate) fn begin_stopping(&self) {
        self.stopping.store(true, Ordering::SeqCst);
        if let Some(session) = self.current_session() {
            session.close();
        }
    }

    /// Stores `needcfg` and returns the value it replaced.
    pub(crate) fn swap_needcfg(&self, needcfg: bool) -> bool {
        self.needcfg.swap(needcfg, Ordering::SeqCst)
    }

    /// The needcfg value the plugin set last.
    pub(crate) fn needcfg(&self) -> bool {
        self.needcfg.load(Ordering::SeqCst)
    }

    /// The needcfg value a `STATUS` carries: always false for 0.9.
    pub(crate) fn status_needcfg(&self) -> bool {
        matches!(self.config.flavour, Flavour::V1_0) && self.needcfg()
    }

    /// True while a session is up.
    pub(crate) fn is_session_up(&self) -> bool {
        self.session_up.load(Ordering::SeqCst)
    }

    /// The session currently up, if any.
    pub(crate) fn current_session(&self) -> Option<Arc<SessionLink>> {
        self.current_session.lock().clone()
    }

    /// When the last game message arrived.
    pub(crate) fn last_game_message(&self) -> Option<Instant> {
        *self.last_game_message.lock()
    }

    /// Records that a game message (a `SYNC_REQ` or an `ASYNC` other than `MISSIONEND`) arrived now.
    pub(crate) fn record_game_message(&self) {
        *self.last_game_message.lock() = Some(Instant::now());
    }

    /// The generation a newly queued async message belongs to.
    pub(crate) fn mission_generation(&self) -> u64 {
        self.mission_generation.load(Ordering::SeqCst)
    }

    /// Starts a new mission generation for a `MISSIONEND`: every queued message becomes stale and
    /// the last game message time is cleared. Returns the new generation.
    pub(crate) fn end_mission(&self) -> u64 {
        *self.last_game_message.lock() = None;
        self.mission_generation
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1)
    }

    /// Handles the text of one async message as an `ASYNC` frame is handled: a `MISSIONEND` starts
    /// a new mission generation, anything else records a game message; then the text joins the
    /// worker's FIFO queue under the current generation. False once the worker has stopped.
    pub(crate) fn queue_async_text(&self, text: Vec<u8>) -> bool {
        let generation = if is_mission_end(&text) {
            self.end_mission()
        } else {
            self.record_game_message();
            self.mission_generation()
        };
        self.async_queue
            .send(AsyncMessage { generation, text })
            .is_ok()
    }

    /// The number of the next accepted connection, unique across every listen port.
    pub(crate) fn next_connection_id(&self) -> u64 {
        self.next_connection_id.fetch_add(1, Ordering::SeqCst)
    }

    /// The address the listen port at `slot` (its place in [`ServerConfig::listen_ports`]) is
    /// bound to.
    pub(crate) fn bound_address(&self, slot: usize) -> Option<SocketAddr> {
        self.bound_addresses.lock().get(slot).copied().flatten()
    }

    /// The addresses of every listen port bound so far, in listen port order.
    pub(crate) fn bound_addresses(&self) -> Vec<SocketAddr> {
        self.bound_addresses
            .lock()
            .iter()
            .flatten()
            .copied()
            .collect()
    }

    /// Records the address the listen port at `slot` is bound to.
    pub(crate) fn set_bound_address(&self, slot: usize, address: SocketAddr) {
        if let Some(bound) = self.bound_addresses.lock().get_mut(slot) {
            *bound = Some(address);
        }
    }

    /// Makes `session` the current one after its `HELLO` passed: closes and reports the session it
    /// replaces, runs `send_greeting` (the answering `HELLO` and first `STATUS`) while pushes are
    /// held back, then reports the new session. Returns false, reporting nothing for it, when the
    /// greeting fails or shutdown has begun.
    pub(crate) fn open_session(
        &self,
        session: &Arc<SessionLink>,
        send_greeting: impl FnOnce(&mut SessionWriter<'_>) -> bool,
    ) -> bool {
        let _order = self.session_order.lock();
        if self.is_stopping() {
            return false;
        }
        let replaced = self.current_session.lock().take();
        if let Some(replaced) = replaced {
            tracing::info!(
                replaced_session = replaced.id(),
                session = session.id(),
                "game link: a new HELLO replaces the open session"
            );
            replaced.close();
            self.session_up.store(false, Ordering::SeqCst);
            self.handler.session_closed();
        }
        let greeted = {
            let mut writer = session.hold_writes();
            *self.current_session.lock() = Some(Arc::clone(session));
            send_greeting(&mut writer)
        };
        if !greeted {
            self.current_session.lock().take();
            return false;
        }
        self.session_up.store(true, Ordering::SeqCst);
        self.handler.session_opened();
        true
    }

    /// Ends `session` when its thread stops: reports it closed unless a newer session replaced it.
    pub(crate) fn close_session(&self, session: &Arc<SessionLink>) {
        session.close();
        let _order = self.session_order.lock();
        let was_current = {
            let mut current = self.current_session.lock();
            if current
                .as_ref()
                .is_some_and(|current| Arc::ptr_eq(current, session))
            {
                current.take();
                true
            } else {
                false
            }
        };
        if was_current {
            self.session_up.store(false, Ordering::SeqCst);
            self.handler.session_closed();
        }
    }
}
