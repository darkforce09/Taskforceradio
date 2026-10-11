//! The error type of `tfar_game_link::server`.

/// Every failure `tfar_game_link::server` reports.
///
/// A busy port is not an error: the server starts, logs it and retries the bind.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operating system refused to start one of the server's threads.
    #[error("the game link could not start its `{thread_name}` thread: {source}")]
    ThreadSpawnFailed {
        /// The name of the thread that did not start.
        thread_name: &'static str,
        /// The error the operating system reported.
        #[source]
        source: std::io::Error,
    },
}

/// The result of a fallible `tfar_game_link::server` operation.
pub type Result<T> = core::result::Result<T, Error>;
