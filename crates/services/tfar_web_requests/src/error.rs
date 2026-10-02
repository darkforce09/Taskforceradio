//! The error type of `tfar_web_requests`.

use std::time::Duration;

/// Every failure `tfar_web_requests` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The request did not finish within its timeout.
    #[error("the request to {url} did not finish within {timeout:?}")]
    TimedOut {
        /// The URL asked for.
        url: String,
        /// The whole-request timeout that ran out.
        timeout: Duration,
    },
    /// The request failed: no connection, a TLS failure, a 4xx or 5xx status, too many
    /// redirects or a body over the size limit.
    #[error("the request to {url} failed: {source}")]
    Request {
        /// The URL asked for.
        url: String,
        /// What `ureq` reported.
        #[source]
        source: Box<ureq::Error>,
    },
    /// A tracking URL could not be built from the text the game sent.
    #[error("`{text}` does not make a valid tracking URL: {source}")]
    InvalidUrl {
        /// The text the URL was built from.
        text: String,
        /// What the URL parser reported.
        #[source]
        source: url::ParseError,
    },
    /// A 1.0 `TRACK` message carried no custom variable the original parser accepts, so the
    /// original sends nothing.
    #[error("the TRACK custom variables `{text}` hold no variable to send")]
    NoCustomVariables {
        /// The custom-variable text the game sent.
        text: String,
    },
    /// The operating system refused to start a worker thread.
    #[error("the `{thread}` thread could not start: {source}")]
    ThreadSpawn {
        /// The name of the thread.
        thread: &'static str,
        /// What the operating system reported.
        #[source]
        source: std::io::Error,
    },
}

/// The result of a fallible `tfar_web_requests` operation.
pub type Result<T> = core::result::Result<T, Error>;
