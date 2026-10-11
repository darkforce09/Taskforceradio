//! The session thread: one connection's handshake, frame reading, sync answers and heartbeat.
//!
//! **Role:** checks the connection's `HELLO`, makes it the current session, then reads frames:
//! answers each `SYNC_REQ` on this thread, queues each `ASYNC` for the worker, forwards `FOCUS`,
//! and sends the `STATUS` heartbeat on time.
//!
//! **Position:** started by the accept loop per accepted connection; calls the plugin's
//! [`GameLinkHandler`](crate::server::GameLinkHandler) and the shared state.
//!
//! **Signals and state:** runs on a `tfar-game-link-session` thread; owns the reading side of the
//! socket and its [`FrameDecoder`]; reads with a timeout no longer than the poll period, so the stop
//! flag, a replacement and the heartbeat are seen on time.
//!
//! **Invariants:** a `HELLO` with another magic, protocol or flavour is logged and closed without a
//! reply; nothing but `HELLO` opens a session; a frame that does not decode closes the session;
//! `STATUS` goes out after the answering `HELLO` and then every `status_interval`; `HELLO` and
//! `FOCUS` never count as game messages and `MISSIONEND` clears the last game message time.

use std::io::{ErrorKind, Read};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::protocol::{Frame, FrameDecoder, Hello};

use crate::server::link_state::LinkState;
use crate::server::session_link::SessionLink;

/// The longest a read blocks before the loop checks the stop flag and the heartbeat.
const READ_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// The shortest read timeout; the socket refuses a zero timeout.
const MINIMUM_READ_TIMEOUT: Duration = Duration::from_millis(1);

/// The size of one socket read.
const READ_CHUNK_SIZE: usize = 16 * 1024;

/// Serves one accepted connection until it closes, fails, is replaced or the server stops.
pub(crate) fn serve_connection(
    state: &Arc<LinkState>,
    stream: TcpStream,
    peer: SocketAddr,
    connection_id: u64,
) {
    if let Err(error) = configure_socket(&stream, state) {
        tracing::warn!(%peer, %error, "game link: cannot configure the accepted socket");
        return;
    }
    let mut reader = FrameReader {
        stream,
        decoder: FrameDecoder::new(),
        peer,
        connection_id,
    };
    let Some(hello) = reader.read_hello(state) else {
        return;
    };
    if let Some(mismatch) = hello.mismatch_with(state.config.flavour) {
        tracing::warn!(
            %peer,
            %mismatch,
            magic = ?hello.magic,
            protocol = hello.protocol,
            flavour = hello.flavour,
            "game link: HELLO refused; closing without a reply"
        );
        return;
    }
    let session = match SessionLink::new(connection_id, &reader.stream) {
        Ok(session) => Arc::new(session),
        Err(error) => {
            tracing::warn!(%peer, %error, "game link: cannot clone the session socket");
            return;
        }
    };
    let flavour = state.config.flavour;
    let opened = state.open_session(&session, |writer| {
        writer.send(&Frame::hello(flavour)) && writer.send_status(state)
    });
    if opened {
        tracing::info!(%peer, session = connection_id, "game link: session up");
        reader.serve_frames(state, &session);
        tracing::info!(%peer, session = connection_id, "game link: session closed");
    }
    state.close_session(&session);
}

/// Sets the socket options of an accepted connection.
fn configure_socket(stream: &TcpStream, state: &LinkState) -> std::io::Result<()> {
    // An accepted socket inherits the listener's non-blocking mode on some systems.
    stream.set_nonblocking(false)?;
    stream.set_nodelay(true)?;
    stream.set_write_timeout(Some(state.config.write_timeout.max(MINIMUM_READ_TIMEOUT)))?;
    stream.set_read_timeout(Some(READ_POLL_INTERVAL))
}

/// What one read of the socket produced.
enum ReadOutcome {
    /// Bytes arrived and are in the decoder.
    Data,
    /// The read timed out with nothing new.
    Idle,
    /// The peer closed the connection, or the socket failed.
    Ended,
}

/// The reading side of one connection.
struct FrameReader {
    /// The accepted socket.
    stream: TcpStream,
    /// Bytes read and not yet decoded.
    decoder: FrameDecoder,
    /// The peer's address, for log lines.
    peer: SocketAddr,
    /// The connection's number, for log lines.
    connection_id: u64,
}

impl FrameReader {
    /// Waits up to `hello_timeout` for the first frame; `None`, logged, unless it is a `HELLO`.
    fn read_hello(&mut self, state: &LinkState) -> Option<Hello> {
        let deadline = Instant::now() + state.config.hello_timeout;
        loop {
            match self.decoder.next_frame() {
                Ok(Some(Frame::Hello(hello))) => return Some(hello),
                Ok(Some(frame)) => {
                    tracing::warn!(peer = %self.peer, kind = %frame.kind(),
                        "game link: first frame is not HELLO; closing");
                    return None;
                }
                Ok(None) => {}
                Err(error) => {
                    tracing::warn!(peer = %self.peer, %error,
                        "game link: malformed frame before HELLO; closing");
                    return None;
                }
            }
            let now = Instant::now();
            if state.is_stopping() || now >= deadline {
                if now >= deadline {
                    tracing::warn!(peer = %self.peer, "game link: no HELLO in time; closing");
                }
                return None;
            }
            if let ReadOutcome::Ended = self.read_some(deadline - now) {
                return None;
            }
        }
    }

    /// Reads and handles frames until the session ends.
    fn serve_frames(&mut self, state: &LinkState, session: &SessionLink) {
        let mut next_status = Instant::now() + state.config.status_interval;
        while !state.is_stopping() && !session.is_closed() {
            match self.decoder.next_frame() {
                Ok(Some(frame)) => {
                    if handle_frame(state, session, frame) {
                        continue;
                    }
                    return;
                }
                Ok(None) => {}
                Err(error) => {
                    tracing::warn!(peer = %self.peer, session = self.connection_id, %error,
                        "game link: malformed frame; closing the session");
                    return;
                }
            }
            let now = Instant::now();
            if now >= next_status {
                if !session.send_status(state) {
                    return;
                }
                next_status = now + state.config.status_interval;
            }
            let until_status = next_status.saturating_duration_since(Instant::now());
            if let ReadOutcome::Ended = self.read_some(until_status) {
                return;
            }
        }
    }

    /// One socket read of at most `wait` (capped at the poll period) into the decoder.
    fn read_some(&mut self, wait: Duration) -> ReadOutcome {
        let timeout = wait.clamp(MINIMUM_READ_TIMEOUT, READ_POLL_INTERVAL);
        if let Err(error) = self.stream.set_read_timeout(Some(timeout)) {
            tracing::warn!(peer = %self.peer, %error, "game link: cannot set the read timeout");
            return ReadOutcome::Ended;
        }
        let mut chunk = [0_u8; READ_CHUNK_SIZE];
        match self.stream.read(&mut chunk) {
            Ok(0) => {
                tracing::debug!(peer = %self.peer, "game link: peer closed the connection");
                ReadOutcome::Ended
            }
            Ok(count) => {
                self.decoder.push(chunk.get(..count).unwrap_or_default());
                ReadOutcome::Data
            }
            Err(error)
                if matches!(
                    error.kind(),
                    ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
                ) =>
            {
                ReadOutcome::Idle
            }
            Err(error) => {
                tracing::debug!(peer = %self.peer, %error, "game link: read failed");
                ReadOutcome::Ended
            }
        }
    }
}

/// Handles one frame of an open session; false when the session must end.
fn handle_frame(state: &LinkState, session: &SessionLink, frame: Frame) -> bool {
    match frame {
        Frame::SyncRequest { id, text } => {
            state.record_game_message();
            let reply = state.handler.sync_request(&text);
            session.send(&Frame::SyncResponse { id, text: reply })
        }
        Frame::Async { text } => {
            if !state.queue_async_text(text) {
                tracing::debug!("game link: async worker stopped; message dropped");
            }
            true
        }
        Frame::Focus { arma_focused } => {
            state.handler.focus_changed(arma_focused);
            true
        }
        other => {
            tracing::debug!(session = session.id(), kind = %other.kind(),
                "game link: frame the plugin does not expect; ignored");
            true
        }
    }
}
