//! Shared helpers of the server's integration tests: a raw-socket client and a recording handler.

#![allow(
    dead_code,
    reason = "each test binary uses a different subset of the helpers"
)]

use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use tfar_game_link::protocol::{Flavour, Frame, FrameDecoder, encode};
use tfar_game_link::server::{GameLinkHandler, GameLinkServer, ServerConfig};

/// How long a test waits for something that should happen at once.
pub(crate) const PROMPT: Duration = Duration::from_secs(2);

/// One callback the server made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    SyncRequest(Vec<u8>),
    Async(Vec<u8>),
    Focus(bool),
    Opened,
    Closed,
}

/// A handler that records every callback, answers `PING` with `PONG` and echoes any other sync
/// call, and can hold the async worker inside a callback.
#[derive(Default)]
pub(crate) struct RecordingHandler {
    events: Mutex<Vec<Event>>,
    changed: Condvar,
    async_gate_closed: Mutex<bool>,
    gate_changed: Condvar,
}

impl RecordingHandler {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub(crate) fn events(&self) -> Vec<Event> {
        self.events.lock().unwrap().clone()
    }

    pub(crate) fn async_texts(&self) -> Vec<Vec<u8>> {
        self.events()
            .into_iter()
            .filter_map(|event| match event {
                Event::Async(text) => Some(text),
                _ => None,
            })
            .collect()
    }

    pub(crate) fn session_events(&self) -> Vec<Event> {
        self.events()
            .into_iter()
            .filter(|event| matches!(event, Event::Opened | Event::Closed))
            .collect()
    }

    /// Waits until `predicate` holds over the events, up to `timeout`; true when it did.
    pub(crate) fn wait_until(
        &self,
        timeout: Duration,
        predicate: impl Fn(&[Event]) -> bool,
    ) -> bool {
        let deadline = Instant::now() + timeout;
        let mut events = self.events.lock().unwrap();
        while !predicate(&events) {
            let now = Instant::now();
            if now >= deadline {
                return false;
            }
            events = self.changed.wait_timeout(events, deadline - now).unwrap().0;
        }
        true
    }

    /// While closed, every async callback waits inside the handler after recording itself.
    pub(crate) fn set_async_gate_closed(&self, closed: bool) {
        *self.async_gate_closed.lock().unwrap() = closed;
        self.gate_changed.notify_all();
    }

    fn record(&self, event: Event) {
        self.events.lock().unwrap().push(event);
        self.changed.notify_all();
    }
}

impl GameLinkHandler for RecordingHandler {
    fn sync_request(&self, text: &[u8]) -> Vec<u8> {
        self.record(Event::SyncRequest(text.to_vec()));
        if text == b"TS_INFO\tPING" {
            b"PONG".to_vec()
        } else {
            [b"ECHO ".as_slice(), text].concat()
        }
    }

    fn async_message(&self, text: &[u8]) {
        self.record(Event::Async(text.to_vec()));
        let mut closed = self.async_gate_closed.lock().unwrap();
        while *closed {
            closed = self.gate_changed.wait(closed).unwrap();
        }
    }

    fn focus_changed(&self, focused: bool) {
        self.record(Event::Focus(focused));
    }

    fn session_opened(&self) {
        self.record(Event::Opened);
    }

    fn session_closed(&self) {
        self.record(Event::Closed);
    }
}

/// A server for `flavour` on an ephemeral loopback port, with its handler and address.
pub(crate) fn start_server(
    flavour: Flavour,
) -> (GameLinkServer, Arc<RecordingHandler>, SocketAddr) {
    start_server_with(ServerConfig::new(flavour, 0))
}

/// A server with `config`, with its handler and address.
pub(crate) fn start_server_with(
    config: ServerConfig,
) -> (GameLinkServer, Arc<RecordingHandler>, SocketAddr) {
    let handler = RecordingHandler::new();
    let server = GameLinkServer::start(config, handler.clone()).unwrap();
    let address = server
        .local_addr()
        .expect("an ephemeral port binds at once");
    (server, handler, address)
}

/// What a raw client saw when it waited for the server to close the connection.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CloseOutcome {
    /// The server closed the connection; these frames came first.
    Closed(Vec<Frame>),
    /// The connection was still open when the wait ended.
    StillOpen,
}

/// A minimal extension stand-in: a raw socket plus the protocol codec, nothing of the client crate.
pub(crate) struct RawClient {
    stream: TcpStream,
    decoder: FrameDecoder,
}

impl RawClient {
    pub(crate) fn connect(address: SocketAddr) -> Self {
        let stream = TcpStream::connect_timeout(&address, PROMPT).unwrap();
        stream.set_nodelay(true).unwrap();
        Self {
            stream,
            decoder: FrameDecoder::new(),
        }
    }

    /// Connects, sends `HELLO` of `flavour` and checks the answering `HELLO` and first `STATUS`;
    /// returns the client and that `STATUS`'s needcfg.
    pub(crate) fn handshake(address: SocketAddr, flavour: Flavour) -> (Self, bool) {
        let mut client = Self::connect(address);
        client.send(&Frame::hello(flavour));
        assert_eq!(client.receive(PROMPT), Some(Frame::hello(flavour)));
        let Some(Frame::Status { needcfg }) = client.receive(PROMPT) else {
            panic!("the answering HELLO is followed by a STATUS");
        };
        (client, needcfg)
    }

    /// Sends a `SYNC_REQ` and returns the next frame that is not a `STATUS`.
    pub(crate) fn sync_call(&mut self, id: u32, text: &[u8]) -> Frame {
        self.send(&Frame::SyncRequest {
            id,
            text: text.to_vec(),
        });
        self.receive_skipping_status(PROMPT)
            .expect("a SYNC_RESP arrives")
    }

    /// Sends one `ASYNC` with `text`.
    pub(crate) fn send_async(&mut self, text: &[u8]) {
        self.send(&Frame::Async {
            text: text.to_vec(),
        });
    }

    pub(crate) fn send(&mut self, frame: &Frame) {
        let mut bytes = Vec::new();
        encode(frame, &mut bytes).unwrap();
        self.send_raw(&bytes);
    }

    pub(crate) fn send_raw(&mut self, bytes: &[u8]) {
        self.stream.write_all(bytes).unwrap();
    }

    /// The next frame within `timeout`; `None` on timeout or close.
    pub(crate) fn receive(&mut self, timeout: Duration) -> Option<Frame> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(frame) = self.decoder.next_frame().unwrap() {
                return Some(frame);
            }
            if self.read_some(deadline)? == 0 {
                return None;
            }
        }
    }

    /// The next frame that is not a `STATUS`, within `timeout`.
    pub(crate) fn receive_skipping_status(&mut self, timeout: Duration) -> Option<Frame> {
        let deadline = Instant::now() + timeout;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match self.receive(left)? {
                Frame::Status { .. } => {}
                frame => return Some(frame),
            }
        }
    }

    /// Reads until the server closes the connection or `timeout` passes.
    pub(crate) fn wait_for_close(&mut self, timeout: Duration) -> CloseOutcome {
        let deadline = Instant::now() + timeout;
        let mut frames = Vec::new();
        loop {
            while let Ok(Some(frame)) = self.decoder.next_frame() {
                frames.push(frame);
            }
            match self.read_some(deadline) {
                Some(0) => return CloseOutcome::Closed(frames),
                Some(_) => {}
                None => return CloseOutcome::StillOpen,
            }
        }
    }

    /// One read before `deadline`: `Some(0)` on close or reset, `Some(n)` for n bytes, `None` on
    /// timeout.
    fn read_some(&mut self, deadline: Instant) -> Option<usize> {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return None;
        }
        self.stream.set_read_timeout(Some(left)).unwrap();
        let mut chunk = [0_u8; 4096];
        match self.stream.read(&mut chunk) {
            Ok(count) => {
                self.decoder.push(&chunk[..count]);
                Some(count)
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                None
            }
            Err(error)
                if matches!(
                    error.kind(),
                    ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted
                ) =>
            {
                Some(0)
            }
            Err(error) => panic!("unexpected read error: {error}"),
        }
    }
}
