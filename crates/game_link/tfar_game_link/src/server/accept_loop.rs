//! The accept threads: one per listen port, each binding its loopback port, retrying it while busy
//! and starting a session thread per connection.
//!
//! **Role:** owns one port's listener; turns each accepted connection into a
//! `tfar-game-link-session` thread and joins those threads before it returns.
//!
//! **Position:** started by [`GameLinkServer::start`](crate::server::GameLinkServer::start), once per
//! [`ListenPort`], with the listener of its first bind attempt; hands connections to the session
//! loop, whose sessions of every port share the one session slot.
//!
//! **Signals and state:** runs on a `tfar-game-link-accept` thread (`tfar-game-link-accept-debug`
//! for the debug port); the listener is non-blocking and polled, so the stop flag is seen within
//! one poll period.
//!
//! **Invariants:** binds `127.0.0.1` only, never another interface; a failed bind is logged and
//! retried every `bind_retry_interval` (5 s by default), independently per port, while that port
//! is dormant; every session thread is joined before its accept thread ends.

use std::io::ErrorKind;
use std::net::{Ipv4Addr, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::server::link_state::LinkState;
use crate::server::session_loop::serve_connection;

/// How often an accept thread polls its listener and the stop flag.
const ACCEPT_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// The name of each connection's thread.
const SESSION_THREAD_NAME: &str = "tfar-game-link-session";

/// One port the server listens on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ListenPort {
    /// The place of the port in [`ServerConfig::listen_ports`](crate::server::ServerConfig::listen_ports):
    /// 0 for the normal port, 1 for the debug port.
    pub(crate) slot: usize,
    /// The port number; 0 lets the system pick one.
    pub(crate) port: u16,
}

/// One bind attempt on `127.0.0.1` and `listen_port`; logs a failure. `is_first_attempt` logs it
/// as a warning, later attempts at debug level.
pub(crate) fn bind_listener(
    state: &LinkState,
    listen_port: ListenPort,
    is_first_attempt: bool,
) -> Option<TcpListener> {
    let port = listen_port.port;
    let bound = TcpListener::bind((Ipv4Addr::LOCALHOST, port))
        .and_then(|listener| listener.set_nonblocking(true).map(|()| listener));
    match bound {
        Ok(listener) => {
            match listener.local_addr() {
                Ok(address) => {
                    state.set_bound_address(listen_port.slot, address);
                    tracing::info!(%address, flavour = %state.config.flavour, "game link: listening");
                }
                Err(error) => tracing::warn!(%error, "game link: bound address unknown"),
            }
            Some(listener)
        }
        Err(error) => {
            let retry_seconds = state.config.bind_retry_interval.as_secs_f64();
            if is_first_attempt {
                tracing::warn!(
                    port,
                    %error,
                    retry_seconds,
                    "game link: cannot listen on 127.0.0.1 (port in use?); dormant, retrying"
                );
            } else {
                tracing::debug!(port, %error, "game link: bind retry failed");
            }
            None
        }
    }
}

/// The accept loop of `listen_port`; `listener` is the result of its first bind attempt.
pub(crate) fn run_accept_loop(
    state: &Arc<LinkState>,
    listen_port: ListenPort,
    mut listener: Option<TcpListener>,
) {
    let mut sessions: Vec<JoinHandle<()>> = Vec::new();
    let mut next_bind_attempt = Instant::now() + state.config.bind_retry_interval;
    while !state.is_stopping() {
        join_finished(&mut sessions);
        let Some(bound) = listener.as_ref() else {
            if Instant::now() >= next_bind_attempt {
                listener = bind_listener(state, listen_port, false);
                next_bind_attempt = Instant::now() + state.config.bind_retry_interval;
            }
            thread::sleep(ACCEPT_POLL_INTERVAL);
            continue;
        };
        match bound.accept() {
            Ok((stream, peer)) => {
                let connection_id = state.next_connection_id();
                let session_state = Arc::clone(state);
                let spawned = thread::Builder::new()
                    .name(SESSION_THREAD_NAME.to_owned())
                    .spawn(move || serve_connection(&session_state, stream, peer, connection_id));
                match spawned {
                    Ok(handle) => sessions.push(handle),
                    Err(error) => {
                        tracing::error!(%peer, %error, "game link: cannot start a session thread");
                    }
                }
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(ACCEPT_POLL_INTERVAL);
            }
            Err(error) => {
                tracing::warn!(%error, "game link: accept failed");
                thread::sleep(ACCEPT_POLL_INTERVAL);
            }
        }
    }
    drop(listener);
    for session in sessions {
        join_session(session);
    }
}

/// Joins the session threads that have already ended.
fn join_finished(sessions: &mut Vec<JoinHandle<()>>) {
    let (finished, running): (Vec<_>, Vec<_>) =
        sessions.drain(..).partition(JoinHandle::is_finished);
    *sessions = running;
    for session in finished {
        join_session(session);
    }
}

/// Joins one session thread, logging one that ended by panicking.
fn join_session(session: JoinHandle<()>) {
    if session.join().is_err() {
        tracing::error!("game link: a session thread panicked");
    }
}
