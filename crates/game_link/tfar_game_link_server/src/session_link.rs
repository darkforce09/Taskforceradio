//! The writing side of one session's socket, shared by its session thread and `set_needcfg`.
//!
//! **Role:** serialises every frame the server writes to one connection and closes the connection
//! when a write fails.
//!
//! **Position:** created by the session thread after a valid `HELLO`; written by that thread
//! (`HELLO`, `STATUS`, `SYNC_RESP`) and by whichever thread calls
//! [`GameLinkServer::set_needcfg`](crate::GameLinkServer::set_needcfg); closed by shutdown and by
//! a replacing session.
//!
//! **Signals and state:** a `parking_lot` lock around one clone of the socket, held for one frame
//! at a time (or the handshake's two); a second, unlocked clone for the shutdown call; the closed
//! flag.
//!
//! **Invariants:** frames never interleave; a failed or timed-out write (the socket's write
//! timeout, 1000 ms by default) closes the socket in both directions, so the session thread's next
//! read ends the session; close never waits for a blocked writer; nothing is written after close.

use std::io;
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::{Mutex, MutexGuard};
use tfar_game_link_protocol::{Error as ProtocolError, Frame, write_frame};

use crate::link_state::LinkState;

/// One session's socket for writing; see the module header.
pub(crate) struct SessionLink {
    /// Counts up per accepted connection, for log lines.
    id: u64,
    /// A clone of the session's socket, used only for writes.
    writer: Mutex<TcpStream>,
    /// A clone used only for `shutdown`, which acts on the shared socket without the write lock.
    closer: TcpStream,
    /// Set by the first close.
    closed: AtomicBool,
}

impl SessionLink {
    /// The writing side of connection `id`, over two clones of `stream`.
    ///
    /// # Errors
    ///
    /// The error of `try_clone` when the system cannot duplicate the socket handle.
    pub(crate) fn new(id: u64, stream: &TcpStream) -> io::Result<Self> {
        Ok(Self {
            id,
            writer: Mutex::new(stream.try_clone()?),
            closer: stream.try_clone()?,
            closed: AtomicBool::new(false),
        })
    }

    /// The connection's number, for log lines.
    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    /// True once the session was closed by any thread.
    pub(crate) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    /// Closes the socket in both directions; a blocked read or write on it returns at once.
    pub(crate) fn close(&self) {
        if !self.closed.swap(true, Ordering::SeqCst)
            && let Err(error) = self.closer.shutdown(Shutdown::Both)
        {
            // The peer may have reset the socket already; it is closed either way.
            tracing::debug!(session = self.id, %error, "game link: socket shutdown failed");
        }
    }

    /// Takes the write lock, holding back every other writer until the guard drops.
    pub(crate) fn hold_writes(&self) -> SessionWriter<'_> {
        SessionWriter {
            link: self,
            stream: self.writer.lock(),
        }
    }

    /// Writes one frame; false when the session is closed or the write failed and closed it.
    pub(crate) fn send(&self, frame: &Frame) -> bool {
        self.hold_writes().send(frame)
    }

    /// Writes one `STATUS` with the needcfg value current once the write lock is held, so the last
    /// `STATUS` on the wire always carries the latest value.
    pub(crate) fn send_status(&self, state: &LinkState) -> bool {
        self.hold_writes().send_status(state)
    }
}

/// The held write lock of one session.
pub(crate) struct SessionWriter<'link> {
    /// The session written to.
    link: &'link SessionLink,
    /// The locked socket clone.
    stream: MutexGuard<'link, TcpStream>,
}

impl SessionWriter<'_> {
    /// Writes one frame; false when the session is closed or the write failed and closed it.
    ///
    /// A frame that cannot be encoded (a reply past the 1 MiB frame limit) is logged and skipped;
    /// the session stays up and the extension's wait for it times out.
    pub(crate) fn send(&mut self, frame: &Frame) -> bool {
        if self.link.is_closed() {
            return false;
        }
        match write_frame(&mut *self.stream, frame) {
            Ok(()) => true,
            Err(ProtocolError::Io { source }) => {
                tracing::warn!(
                    session = self.link.id,
                    kind = %frame.kind(),
                    error = %source,
                    "game link: write failed or timed out; closing the session"
                );
                self.link.close();
                false
            }
            Err(error) => {
                tracing::error!(
                    session = self.link.id,
                    kind = %frame.kind(),
                    %error,
                    "game link: frame not sent"
                );
                true
            }
        }
    }

    /// Writes one `STATUS` carrying the needcfg value current now.
    pub(crate) fn send_status(&mut self, state: &LinkState) -> bool {
        let needcfg = state.status_needcfg();
        self.send(&Frame::Status { needcfg })
    }
}
