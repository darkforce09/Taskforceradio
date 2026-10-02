//! One connected, handshaken session with the plugin.
//!
//! **Role:** opens a session (TCP connect within the connect timeout, `HELLO` out, the plugin's
//! `HELLO` back within the `HELLO` timeout), starts its reader thread, writes frames and waits for
//! the `SYNC_RESP` of one request id.
//!
//! **Position:** owned by [`GameLinkClient`](crate::GameLinkClient); calls
//! `tfar_game_link_protocol`'s codec and hands the socket's second handle to
//! [`session_reader`](crate::session_reader).
//!
//! **Signals and state:** the write handle and the response channel are used from Arma's calling
//! thread under the client's mutex; [`SessionLiveness`] is shared with the reader thread through
//! atomics. Dropping a session tells the reader to stop and shuts the socket down.
//!
//! **Invariants:** request ids count up from 1 per session and never use 0; a response with another
//! id than the awaited one is discarded; the session is live only while the reader has not seen the
//! connection end and the last plugin frame is younger than the dead-session time.

use core::time::Duration;
use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::Instant;

use tfar_game_link_protocol::{Frame, FrameDecoder, encode};

use crate::client_config::ClientConfig;
use crate::error::{Error, Result};
use crate::session_reader::{SyncResponse, spawn_session_reader};

/// The size of one socket read, while waiting for `HELLO` and in the reader thread.
pub(crate) const READ_CHUNK_SIZE: usize = 16 * 1024;

/// What the reader thread publishes about a session, read lock-free by the calling thread.
#[derive(Debug)]
pub(crate) struct SessionLiveness {
    /// The instant the other times count from.
    epoch: Instant,
    /// Milliseconds after `epoch` of the latest frame from the plugin.
    last_frame_millis: AtomicU64,
    /// Milliseconds after `epoch` at which the reader saw the connection end.
    closed_millis: AtomicU64,
    /// Set by the reader when the connection ended or a frame did not decode.
    is_closed: AtomicBool,
    /// Set by the owner when the session is dropped, so the reader stops at its next poll.
    is_stop_requested: AtomicBool,
}

impl SessionLiveness {
    fn new() -> Self {
        Self {
            epoch: Instant::now(),
            last_frame_millis: AtomicU64::new(0),
            closed_millis: AtomicU64::new(0),
            is_closed: AtomicBool::new(false),
            is_stop_requested: AtomicBool::new(false),
        }
    }

    /// Records that a frame from the plugin arrived now.
    pub(crate) fn record_frame(&self) {
        self.last_frame_millis
            .store(self.millis_since_epoch(), Ordering::Release);
    }

    fn millis_since_epoch(&self) -> u64 {
        u64::try_from(self.epoch.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    /// The time since the latest frame from the plugin.
    pub(crate) fn since_last_frame(&self) -> Duration {
        let last = Duration::from_millis(self.last_frame_millis.load(Ordering::Acquire));
        self.epoch.elapsed().saturating_sub(last)
    }

    /// Records that the connection ended; the reader calls it before it exits.
    pub(crate) fn mark_closed(&self) {
        self.closed_millis
            .store(self.millis_since_epoch(), Ordering::Release);
        self.is_closed.store(true, Ordering::Release);
    }

    /// Whether the reader has seen the connection end.
    pub(crate) fn is_closed(&self) -> bool {
        self.is_closed.load(Ordering::Acquire)
    }

    /// Whether the owner has dropped the session.
    pub(crate) fn is_stop_requested(&self) -> bool {
        self.is_stop_requested.load(Ordering::Acquire)
    }
}

/// How waiting for one request's `SYNC_RESP` ended.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ResponseWait {
    /// The response with the awaited id arrived.
    Reply(Vec<u8>),
    /// The sync timeout passed first.
    TimedOut,
    /// The connection ended first.
    Closed,
}

/// One connected session: the write handle, the reader's response channel and its liveness.
#[derive(Debug)]
pub(crate) struct Session {
    stream: TcpStream,
    liveness: Arc<SessionLiveness>,
    responses: Receiver<SyncResponse>,
    next_request_id: u32,
}

impl Session {
    /// Connects to the plugin, completes the handshake and starts the reader thread.
    ///
    /// The plugin's `STATUS` frames update `needcfg`, which the caller keeps across sessions.
    pub(crate) fn open(config: &ClientConfig, needcfg: Arc<AtomicBool>) -> Result<Self> {
        let port = config.port;
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        let mut stream = TcpStream::connect_timeout(&address, config.connect_timeout)
            .map_err(|source| Error::Connect { port, source })?;
        let setup = |operation: &'static str| {
            move |source| Error::SocketSetup {
                port,
                operation,
                source,
            }
        };
        stream.set_nodelay(true).map_err(setup("set_nodelay"))?;
        stream
            .set_write_timeout(Some(config.write_timeout))
            .map_err(setup("set_write_timeout"))?;
        write_frame_to(&mut stream, &Frame::hello(config.flavour))?;

        let mut decoder = FrameDecoder::new();
        await_plugin_hello(&mut stream, &mut decoder, config)?;

        let liveness = Arc::new(SessionLiveness::new());
        liveness.record_frame();
        let reader_stream = stream.try_clone().map_err(setup("try_clone"))?;
        let (sender, responses) = mpsc::channel();
        spawn_session_reader(
            reader_stream,
            decoder,
            Arc::clone(&liveness),
            sender,
            needcfg,
        )
        .map_err(|source| Error::ReaderThread { source })?;
        Ok(Self {
            stream,
            liveness,
            responses,
            next_request_id: 1,
        })
    }

    /// Whether the session can carry a call: the reader has not seen the connection end and a
    /// plugin frame arrived within `dead_after`.
    pub(crate) fn is_live(&self, dead_after: Duration) -> bool {
        !self.liveness.is_closed() && self.liveness.since_last_frame() <= dead_after
    }

    /// When the session stopped being usable: the instant the reader saw the connection end, or
    /// the instant the dead-session time ran out after the last plugin frame; `None` while it is
    /// live.
    pub(crate) fn ended_at(&self, dead_after: Duration) -> Option<Instant> {
        let liveness = &self.liveness;
        if liveness.is_closed() {
            let closed = liveness.closed_millis.load(Ordering::Acquire);
            return Some(liveness.epoch + Duration::from_millis(closed));
        }
        let last_frame = liveness.last_frame_millis.load(Ordering::Acquire);
        let dead_at = liveness.epoch + Duration::from_millis(last_frame) + dead_after;
        (dead_at < Instant::now()).then_some(dead_at)
    }

    /// The id for the next `SYNC_REQ`: 1, 2, 3 … per session, skipping 0 if it ever wraps.
    pub(crate) fn take_request_id(&mut self) -> u32 {
        let id = self.next_request_id;
        self.next_request_id = match id.wrapping_add(1) {
            0 => 1,
            next => next,
        };
        id
    }

    /// Writes one frame, bounded by the write timeout set when the session opened.
    pub(crate) fn send(&mut self, frame: &Frame) -> Result<()> {
        write_frame_to(&mut self.stream, frame)
    }

    /// Waits up to `timeout` for the `SYNC_RESP` with `id`, discarding responses of other ids.
    pub(crate) fn wait_for_response(&self, id: u32, timeout: Duration) -> ResponseWait {
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.responses.recv_timeout(remaining) {
                Ok(response) if response.id == id => return ResponseWait::Reply(response.text),
                Ok(response) => {
                    tracing::debug!(
                        stale_id = response.id,
                        awaited_id = id,
                        "discarding a game link SYNC_RESP for an earlier request"
                    );
                }
                Err(RecvTimeoutError::Timeout) => return ResponseWait::TimedOut,
                Err(RecvTimeoutError::Disconnected) => return ResponseWait::Closed,
            }
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.liveness
            .is_stop_requested
            .store(true, Ordering::Release);
        // The peer may already be gone; a failed shutdown leaves nothing to undo.
        let _ = self.stream.shutdown(Shutdown::Both);
    }
}

/// Encodes `frame` and writes it whole.
fn write_frame_to(stream: &mut TcpStream, frame: &Frame) -> Result<()> {
    let mut bytes = Vec::new();
    encode(frame, &mut bytes)?;
    stream
        .write_all(&bytes)
        .and_then(|()| stream.flush())
        .map_err(|source| Error::Write {
            kind: frame.kind(),
            source,
        })
}

/// Reads until the plugin's `HELLO` arrives and matches, within the `HELLO` timeout.
///
/// Bytes after the `HELLO` (the plugin's first `STATUS`) stay in `decoder` for the reader thread.
fn await_plugin_hello(
    stream: &mut TcpStream,
    decoder: &mut FrameDecoder,
    config: &ClientConfig,
) -> Result<()> {
    let port = config.port;
    let timeout = config.hello_timeout;
    let deadline = Instant::now() + timeout;
    let mut chunk = vec![0_u8; READ_CHUNK_SIZE];
    loop {
        if let Some(frame) = decoder.next_frame()? {
            return match frame {
                Frame::Hello(hello) => match hello.mismatch_with(config.flavour) {
                    None => Ok(()),
                    Some(mismatch) => Err(Error::HelloRefused { port, mismatch }),
                },
                other => Err(Error::FrameBeforeHello {
                    port,
                    kind: other.kind(),
                }),
            };
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(Error::HelloTimeout { port, timeout });
        }
        stream
            .set_read_timeout(Some(remaining))
            .map_err(|source| Error::SocketSetup {
                port,
                operation: "set_read_timeout",
                source,
            })?;
        match stream.read(&mut chunk) {
            Ok(0) => return Err(Error::ClosedDuringHandshake { port }),
            Ok(count) => decoder.push(chunk.get(..count).unwrap_or_default()),
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                return Err(Error::HelloTimeout { port, timeout });
            }
            Err(_) => return Err(Error::ClosedDuringHandshake { port }),
        }
    }
}
