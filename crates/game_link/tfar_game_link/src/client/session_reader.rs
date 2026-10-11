//! The reader thread of one session: decodes the plugin's frames and publishes what they say.
//!
//! **Role:** reads the session socket, decodes frames with `tfar_game_link::protocol`'s
//! `FrameDecoder`, stores every `STATUS` needcfg, records each frame's arrival time and passes
//! `SYNC_RESP` frames to the calling thread through a channel.
//!
//! **Position:** started by [`Session::open`](crate::client::session::Session::open) with the socket's
//! second handle and the bytes already buffered after the plugin's `HELLO`.
//!
//! **Signals and state:** one thread per session, named `tfar_game_link_client_reader`; writes the
//! client's needcfg atomic and the session's [`SessionLiveness`]. It polls the socket with a short
//! read timeout, so it ends soon after the owner drops the session even if the shutdown does not
//! wake a blocked read.
//!
//! **Invariants:** the reader marks the session closed before it drops the channel's sender, so a
//! caller that sees the channel disconnect also sees the session closed; a frame only the extension
//! sends (`SYNC_REQ`, `ASYNC`, `FOCUS`) or a frame that does not decode ends the session.

use core::time::Duration;
use std::io::{ErrorKind, Read};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread;

use crate::protocol::{Frame, FrameDecoder};

use crate::client::session::{READ_CHUNK_SIZE, SessionLiveness};

/// How often a blocked read wakes to check whether the owner dropped the session.
const STOP_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// One `SYNC_RESP` passed from the reader to the calling thread.
#[derive(Debug)]
pub(crate) struct SyncResponse {
    /// The id of the request it answers.
    pub(crate) id: u32,
    /// The reply text, byte for byte.
    pub(crate) text: Vec<u8>,
}

/// Starts the reader thread of one session.
pub(crate) fn spawn_session_reader(
    stream: TcpStream,
    decoder: FrameDecoder,
    liveness: Arc<SessionLiveness>,
    responses: Sender<SyncResponse>,
    needcfg: Arc<AtomicBool>,
) -> std::io::Result<()> {
    stream.set_read_timeout(Some(STOP_POLL_INTERVAL))?;
    let reader = SessionReader {
        stream,
        decoder,
        liveness,
        responses,
        needcfg,
    };
    thread::Builder::new()
        .name("tfar_game_link_client_reader".to_owned())
        .spawn(move || reader.run())
        .map(drop)
}

/// What the reader thread owns.
struct SessionReader {
    stream: TcpStream,
    decoder: FrameDecoder,
    liveness: Arc<SessionLiveness>,
    responses: Sender<SyncResponse>,
    needcfg: Arc<AtomicBool>,
}

impl SessionReader {
    /// Reads and handles frames until the connection ends, a frame is refused or the owner stops it.
    fn run(mut self) {
        let mut chunk = vec![0_u8; READ_CHUNK_SIZE];
        loop {
            if !self.handle_buffered_frames() || self.liveness.is_stop_requested() {
                break;
            }
            match self.stream.read(&mut chunk) {
                Ok(0) => {
                    tracing::info!("the game link plugin closed the connection");
                    break;
                }
                Ok(count) => self.decoder.push(chunk.get(..count).unwrap_or_default()),
                Err(error)
                    if matches!(
                        error.kind(),
                        ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
                    ) => {}
                Err(error) => {
                    tracing::info!(%error, "reading from the game link plugin failed");
                    break;
                }
            }
        }
        self.liveness.mark_closed();
    }

    /// Handles every complete frame in the decoder; `false` when the session must end.
    fn handle_buffered_frames(&mut self) -> bool {
        loop {
            match self.decoder.next_frame() {
                Ok(Some(frame)) => {
                    self.liveness.record_frame();
                    if !self.handle_frame(frame) {
                        return false;
                    }
                }
                Ok(None) => return true,
                Err(error) => {
                    tracing::warn!(%error, "closing the game link session: a plugin frame does not decode");
                    return false;
                }
            }
        }
    }

    /// Handles one plugin frame; `false` when the frame ends the session.
    fn handle_frame(&self, frame: Frame) -> bool {
        match frame {
            Frame::Status { needcfg } => {
                self.needcfg.store(needcfg, Ordering::Release);
                true
            }
            Frame::SyncResponse { id, text } => {
                // A send fails only when the session was dropped, which ends the reader anyway.
                let _ = self.responses.send(SyncResponse { id, text });
                true
            }
            // A repeated HELLO carries nothing new; the handshake has already checked the first.
            Frame::Hello(_) => true,
            Frame::SyncRequest { .. } | Frame::Async { .. } | Frame::Focus { .. } => {
                tracing::warn!(kind = %frame.kind(), "closing the game link session: the plugin sent a frame only the extension sends");
                false
            }
        }
    }
}
