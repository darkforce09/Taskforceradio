//! The accept thread: binds the loopback port, retries a busy one and starts a session thread per
//! connection.
//!
//! **Role:** owns the listener; turns each accepted connection into a `tfar-game-link-session`
//! thread and joins those threads before it returns.
//!
//! **Position:** started by [`GameLinkServer::start`](crate::GameLinkServer::start) with the
//! listener of its first bind attempt; hands connections to the session loop.
//!
//! **Signals and state:** runs on the `tfar-game-link-accept` thread; the listener is
//! non-blocking and polled, so the stop flag is seen within one poll period.
//!
//! **Invariants:** binds `127.0.0.1` only, never another interface; a failed bind is logged and
//! retried every `bind_retry_interval` (5 s by default) while the server is otherwise dormant;
//! every session thread is joined before the accept thread ends.

use std::io::ErrorKind;
use std::net::{Ipv4Addr, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::link_state::LinkState;
use crate::session_loop::serve_connection;

/// How often the accept thread polls the listener and the stop flag.
const ACCEPT_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// The name of each connection's thread.
const SESSION_THREAD_NAME: &str = "tfar-game-link-session";

/// One bind attempt on `127.0.0.1` and the configured port; logs a failure. `is_first_attempt`
/// logs it as a warning, later attempts at debug level.
pub(crate) fn bind_listener(state: &LinkState, is_first_attempt: bool) -> Option<TcpListener> {
    let port = state.config.port;
    let bound = TcpListener::bind((Ipv4Addr::LOCALHOST, port))
        .and_then(|listener| listener.set_nonblocking(true).map(|()| listener));
    match bound {
        Ok(listener) => {
            match listener.local_addr() {
                Ok(address) => {
                    state.set_local_addr(address);
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

/// The accept loop; `listener` is the result of the first bind attempt.
pub(crate) fn run_accept_loop(state: &Arc<LinkState>, mut listener: Option<TcpListener>) {
    let mut sessions: Vec<JoinHandle<()>> = Vec::new();
    let mut next_bind_attempt = Instant::now() + state.config.bind_retry_interval;
    let mut next_connection_id: u64 = 1;
    while !state.is_stopping() {
        join_finished(&mut sessions);
        let Some(bound) = listener.as_ref() else {
            if Instant::now() >= next_bind_attempt {
                listener = bind_listener(state, false);
                next_bind_attempt = Instant::now() + state.config.bind_retry_interval;
            }
            thread::sleep(ACCEPT_POLL_INTERVAL);
            continue;
        };
        match bound.accept() {
            Ok((stream, peer)) => {
                let connection_id = next_connection_id;
                next_connection_id = next_connection_id.wrapping_add(1);
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
