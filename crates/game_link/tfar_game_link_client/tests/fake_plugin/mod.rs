//! A scripted stand-in for the plugin side of the game link: a real loopback listener that speaks
//! raw frames through `tfar_game_link_protocol`'s codec, so the client is tested against bytes on a
//! socket and not against `tfar_game_link_server`.

use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use tfar_game_link_protocol::{Flavour, Frame, read_frame, write_frame};

/// The longest a fake plugin waits for one frame, so a broken test fails instead of hanging.
const PLUGIN_READ_TIMEOUT: Duration = Duration::from_secs(5);

/// A listening fake plugin that runs `script` on every accepted connection, in its own thread.
pub(crate) struct FakePlugin {
    port: u16,
    accepts: Arc<AtomicUsize>,
}

impl FakePlugin {
    /// Binds an ephemeral `127.0.0.1` port and serves every connection with `script`.
    pub(crate) fn start(script: impl Fn(PluginConnection) + Send + Sync + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let accepts = Arc::new(AtomicUsize::new(0));
        let accept_count = Arc::clone(&accepts);
        let script = Arc::new(script);
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { return };
                let index = accept_count.fetch_add(1, Ordering::SeqCst);
                let script = Arc::clone(&script);
                thread::spawn(move || script(PluginConnection::new(stream, index)));
            }
        });
        Self { port, accepts }
    }

    /// The port the fake plugin listens on.
    pub(crate) fn port(&self) -> u16 {
        self.port
    }

    /// How many connections the fake plugin has accepted.
    pub(crate) fn accepts(&self) -> usize {
        self.accepts.load(Ordering::SeqCst)
    }
}

/// One accepted connection, seen from the plugin side.
pub(crate) struct PluginConnection {
    stream: TcpStream,
    /// 0 for the first connection the plugin accepted, 1 for the second, and so on.
    pub(crate) index: usize,
}

impl PluginConnection {
    fn new(stream: TcpStream, index: usize) -> Self {
        stream.set_nodelay(true).unwrap();
        stream.set_read_timeout(Some(PLUGIN_READ_TIMEOUT)).unwrap();
        Self { stream, index }
    }

    /// Reads the next frame from the extension.
    pub(crate) fn read(&mut self) -> Frame {
        read_frame(&mut self.stream).unwrap()
    }

    /// Reads the next frame, or `None` when the extension closed the connection or nothing came.
    pub(crate) fn try_read(&mut self) -> Option<Frame> {
        read_frame(&mut self.stream).ok()
    }

    /// Writes one frame to the extension.
    pub(crate) fn write(&mut self, frame: &Frame) {
        write_frame(&mut self.stream, frame).unwrap();
    }

    /// Reads the extension's `HELLO`, checks it names `flavour`, and answers `HELLO` and one
    /// `STATUS` with `needcfg`, as the plugin does.
    pub(crate) fn accept_handshake(&mut self, flavour: Flavour, needcfg: bool) {
        match self.read() {
            Frame::Hello(hello) => assert_eq!(hello.mismatch_with(flavour), None),
            other => panic!("expected HELLO, got {other:?}"),
        }
        self.write(&Frame::hello(flavour));
        self.write(&Frame::Status { needcfg });
    }

    /// Reads the next `SYNC_REQ` and returns its id and text.
    pub(crate) fn read_sync_request(&mut self) -> (u32, Vec<u8>) {
        match self.read() {
            Frame::SyncRequest { id, text } => (id, text),
            other => panic!("expected SYNC_REQ, got {other:?}"),
        }
    }

    /// Sends `STATUS` every `interval` from a second thread for as long as the connection is open.
    pub(crate) fn start_heartbeat(&self, interval: Duration) {
        let mut stream = self.stream.try_clone().unwrap();
        thread::spawn(move || {
            while write_frame(&mut stream, &Frame::Status { needcfg: false }).is_ok() {
                thread::sleep(interval);
            }
        });
    }

    /// Keeps the connection open, sending nothing, until the extension closes it or `limit` passes.
    pub(crate) fn hold_silently(mut self, limit: Duration) {
        self.stream.set_read_timeout(Some(limit)).unwrap();
        while self.try_read().is_some() {}
    }
}

/// A port nothing listens on: bound once to get a free port, then released.
pub(crate) fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
