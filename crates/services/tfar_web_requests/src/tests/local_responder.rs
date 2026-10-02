//! A loopback HTTP responder for the tests: a `TcpListener` on 127.0.0.1 that answers one scripted
//! reply per connection and records each request line. No test reaches the real network.

use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;
use std::time::Duration;

/// What the responder does with one connection.
#[derive(Clone, Debug)]
pub(crate) enum ScriptedReply {
    /// `200 OK` with this body.
    Body(&'static str),
    /// `301 Moved Permanently` to this path on the same responder.
    RedirectTo(&'static str),
    /// `404 Not Found` with a short HTML body.
    NotFound,
    /// Reads the request, then says nothing for this long before closing.
    Silence(Duration),
}

/// The responder; its thread ends after the last scripted reply.
pub(crate) struct LocalResponder {
    address: SocketAddr,
    request_lines: Receiver<String>,
    thread: Option<JoinHandle<()>>,
}

impl LocalResponder {
    /// Binds an ephemeral loopback port and serves `replies`, one connection each, in order.
    pub(crate) fn start(replies: Vec<ScriptedReply>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a loopback port");
        let address = listener.local_addr().expect("read the bound address");
        let (sender, request_lines) = channel();
        let thread = std::thread::spawn(move || serve(&listener, address, &replies, &sender));
        Self {
            address,
            request_lines,
            thread: Some(thread),
        }
    }

    /// The `http://127.0.0.1:<port><path>` URL of `path` on this responder.
    pub(crate) fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.address)
    }

    /// The next request line received (`GET /path?query HTTP/1.1`), waiting up to two seconds.
    pub(crate) fn next_request_line(&self) -> String {
        self.request_lines
            .recv_timeout(Duration::from_secs(2))
            .expect("a request reached the responder")
    }
}

impl Drop for LocalResponder {
    fn drop(&mut self) {
        // A test that sent fewer requests than scripted leaves the thread in `accept`; it ends
        // with the test process, so it is detached rather than joined.
        if let Some(thread) = self.thread.take()
            && thread.is_finished()
        {
            let _ = thread.join();
        }
    }
}

fn serve(
    listener: &TcpListener,
    address: SocketAddr,
    replies: &[ScriptedReply],
    request_lines: &Sender<String>,
) {
    for reply in replies {
        let Ok((stream, _)) = listener.accept() else {
            return;
        };
        answer(stream, address, reply, request_lines);
    }
}

fn answer(
    mut stream: TcpStream,
    address: SocketAddr,
    reply: &ScriptedReply,
    request_lines: &Sender<String>,
) {
    let mut reader = BufReader::new(stream.try_clone().expect("clone the stream"));
    let mut request_line = String::new();
    let _ = reader.read_line(&mut request_line);
    let _ = request_lines.send(request_line.trim_end().to_owned());
    loop {
        let mut header = String::new();
        match reader.read_line(&mut header) {
            Ok(0) | Err(_) => break,
            Ok(_) if header == "\r\n" || header == "\n" => break,
            Ok(_) => {}
        }
    }
    let response = match reply {
        ScriptedReply::Body(body) => format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
        ScriptedReply::RedirectTo(path) => format!(
            "HTTP/1.1 301 Moved Permanently\r\nLocation: http://{address}{path}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        ),
        ScriptedReply::NotFound => {
            let body = "<html><body>404</body></html>";
            format!(
                "HTTP/1.1 404 Not Found\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
        }
        ScriptedReply::Silence(duration) => {
            std::thread::sleep(*duration);
            return;
        }
    };
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}
