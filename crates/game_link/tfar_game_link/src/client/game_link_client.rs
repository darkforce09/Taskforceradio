//! The extension's handle on the game link: lazy connect, backoff, sync, async and focus calls.
//!
//! **Role:** [`GameLinkClient`] owns at most one [`Session`], opens it on the first call that needs
//! one, drops it when it closes, goes quiet or a write fails, and turns each call into a
//! [`SyncOutcome`] or a [`Result`]. It is policy-free: which string Arma sees is the extension
//! app's choice.
//!
//! **Position:** called by `tfar_extension_0_9` and `tfar_extension_1_0` from `RVExtension`, and by
//! `tfar_ts3_mock`; calls [`Session`].
//!
//! **Signals and state:** the session, the backoff deadline and the latest focus value behind one
//! mutex, taken by Arma's calling thread for the length of a call; the needcfg flag in an atomic the
//! reader threads write.
//!
//! **Invariants:** no connect is attempted while the backoff runs (the backoff starts at a failed
//! connect or handshake and at every dropped session); a dead session (no plugin frame within the
//! dead-session time) is dropped before a call uses it, and the backoff of a session that ended
//! between calls counts from the moment it ended; needcfg reads true from the start of each session
//! until the plugin's `STATUS` says otherwise; after a new session's handshake the latest value
//! given to [`GameLinkClient::send_focus`], if any, is sent before anything else.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use crate::protocol::Frame;

use crate::client::client_config::ClientConfig;
use crate::client::error::{Error, Result};
use crate::client::session::{ResponseWait, Session};
use crate::client::sync_outcome::SyncOutcome;

#[cfg(test)]
#[path = "tests/game_link_client.rs"]
mod tests;

/// The extension side of one game link: one lazily opened session with the plugin.
///
/// Every method takes `&self`, so an extension keeps the client in a `static` (`OnceLock`) and calls
/// it from `RVExtension`; calls are serialised by an internal mutex.
#[derive(Debug)]
pub struct GameLinkClient {
    config: ClientConfig,
    needcfg: Arc<AtomicBool>,
    connection: Mutex<ConnectionState>,
}

/// The state behind the client's mutex.
#[derive(Debug, Default)]
struct ConnectionState {
    session: Option<Session>,
    /// No connect attempt before this instant.
    retry_after: Option<Instant>,
    /// The latest value given to `send_focus`, replayed after each new handshake.
    last_focus: Option<bool>,
}

/// A session ready for a call, and whether it was opened by this call.
struct ReadySession<'state> {
    session: &'state mut Session,
    is_new: bool,
}

impl GameLinkClient {
    /// A client for `config`. Nothing connects until the first call that needs a session.
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config,
            needcfg: Arc::new(AtomicBool::new(true)),
            connection: Mutex::new(ConnectionState::default()),
        }
    }

    /// The settings this client was made with.
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Sends `text` as a `SYNC_REQ` and waits up to the sync timeout for its `SYNC_RESP`.
    ///
    /// Opens a session first when none is up and the backoff is not running. A response to an
    /// earlier request (a late answer after a timeout) is discarded.
    pub fn sync_request(&self, text: &[u8]) -> SyncOutcome {
        let mut connection = self.lock_connection();
        let ready = match self.ready_session(&mut connection) {
            Ok(ready) => ready,
            Err(error) => {
                tracing::debug!(%error, "no game link session for a sync request");
                return SyncOutcome::NoSession;
            }
        };
        let id = ready.session.take_request_id();
        let request = Frame::SyncRequest {
            id,
            text: text.to_vec(),
        };
        if let Err(error) = ready.session.send(&request) {
            let outcome = if error.is_write_timeout() {
                SyncOutcome::TimedOut
            } else {
                SyncOutcome::ClosedDuringCall
            };
            self.drop_session(&mut connection, &error);
            return outcome;
        }
        match ready
            .session
            .wait_for_response(id, self.config.sync_timeout)
        {
            ResponseWait::Reply(text) => SyncOutcome::Reply(text),
            ResponseWait::TimedOut => SyncOutcome::TimedOut,
            ResponseWait::Closed => {
                tracing::info!("the game link plugin closed the session during a sync request");
                self.end_session_now(&mut connection);
                SyncOutcome::ClosedDuringCall
            }
        }
    }

    /// Sends `text` as an `ASYNC` frame without waiting for anything.
    ///
    /// Opens a session first when none is up and the backoff is not running.
    ///
    /// # Errors
    ///
    /// The error of the connect or handshake when no session could be opened,
    /// [`Error::BackoffRunning`] while the backoff runs, or the write error, which drops the
    /// session.
    pub fn send_async(&self, text: &[u8]) -> Result<()> {
        let mut connection = self.lock_connection();
        let ready = self.ready_session(&mut connection)?;
        let sent = ready.session.send(&Frame::Async {
            text: text.to_vec(),
        });
        if let Err(error) = &sent {
            self.drop_session(&mut connection, error);
        }
        sent
    }

    /// Sends a `FOCUS` frame with whether Arma is in front.
    ///
    /// The value is kept and sent again right after every later handshake, so the plugin learns the
    /// state of a new session before its first call. Opens a session first when none is up and the
    /// backoff is not running; a session opened by this call has already carried the value.
    ///
    /// # Errors
    ///
    /// As [`GameLinkClient::send_async`]; the value is kept even when sending fails.
    pub fn send_focus(&self, arma_focused: bool) -> Result<()> {
        let mut connection = self.lock_connection();
        connection.last_focus = Some(arma_focused);
        let ready = self.ready_session(&mut connection)?;
        if ready.is_new {
            return Ok(());
        }
        let sent = ready.session.send(&Frame::Focus { arma_focused });
        if let Err(error) = &sent {
            self.drop_session(&mut connection, error);
        }
        sent
    }

    /// Whether the plugin's latest `STATUS` asked for the configuration (true until the first
    /// `STATUS` of a session arrives).
    pub fn needcfg(&self) -> bool {
        self.needcfg.load(Ordering::Acquire)
    }

    /// Whether a session is up and the plugin sent a frame within the dead-session time. Never
    /// connects.
    pub fn is_live(&self) -> bool {
        self.lock_connection()
            .session
            .as_ref()
            .is_some_and(|session| session.is_live(self.config.session_dead_after))
    }

    /// The live session, opening one when none is up and the backoff allows it.
    fn ready_session<'state>(
        &self,
        connection: &'state mut ConnectionState,
    ) -> Result<ReadySession<'state>> {
        let ended_at = connection
            .session
            .as_ref()
            .and_then(|session| session.ended_at(self.config.session_dead_after));
        if let Some(ended_at) = ended_at {
            tracing::info!("dropping a game link session that closed or went quiet");
            // The backoff counts from the moment the session ended, not from this call.
            connection.session = None;
            connection.retry_after = Some(ended_at + self.config.backoff);
        }
        let is_new = connection.session.is_none();
        if is_new {
            let session = self.open_session(connection)?;
            connection.session = Some(session);
        }
        match connection.session.as_mut() {
            Some(session) => Ok(ReadySession { session, is_new }),
            None => Err(Error::BackoffRunning {
                remaining: self.config.backoff,
            }),
        }
    }

    /// Opens a session unless the backoff runs; a failure starts the backoff.
    fn open_session(&self, connection: &mut ConnectionState) -> Result<Session> {
        let now = Instant::now();
        if let Some(retry_after) = connection.retry_after
            && now < retry_after
        {
            return Err(Error::BackoffRunning {
                remaining: retry_after.saturating_duration_since(now),
            });
        }
        self.needcfg.store(true, Ordering::Release);
        let opened =
            Session::open(&self.config, Arc::clone(&self.needcfg)).and_then(|mut session| {
                match connection.last_focus {
                    Some(arma_focused) => session
                        .send(&Frame::Focus { arma_focused })
                        .map(|()| session),
                    None => Ok(session),
                }
            });
        match opened {
            Ok(session) => {
                connection.retry_after = None;
                Ok(session)
            }
            Err(error) => {
                tracing::debug!(%error, "opening a game link session failed");
                connection.retry_after = Some(Instant::now() + self.config.backoff);
                Err(error)
            }
        }
    }

    /// Drops the session after `error` and starts the backoff.
    fn drop_session(&self, connection: &mut ConnectionState, error: &Error) {
        tracing::info!(%error, "dropping the game link session");
        self.end_session_now(connection);
    }

    /// Drops the session and starts the backoff from now.
    fn end_session_now(&self, connection: &mut ConnectionState) {
        connection.session = None;
        connection.retry_after = Some(Instant::now() + self.config.backoff);
    }

    fn lock_connection(&self) -> MutexGuard<'_, ConnectionState> {
        // Every method leaves the state consistent before it can fail, so a poisoned lock is still
        // usable (and with `panic = "abort"` it never poisons outside tests).
        self.connection
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}
