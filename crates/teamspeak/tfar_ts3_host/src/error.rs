//! The error types of `tfar_ts3_host`: [`Ts3Error`] for a failed TeamSpeak call and [`Error`] for
//! everything else the crate reports.

/// A TeamSpeak function call that failed, or that could not be made.
///
/// Every variant names the TeamSpeak function (`function`, the SDK name such as
/// `getClientVariableAsString`), so a log line says which call to look at.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Ts3Error {
    /// TeamSpeak answered with an error code; `message` is its text from `getErrorMessage`.
    #[error("TeamSpeak `{function}` failed with error {code:#06x}: {message}")]
    Call {
        /// The SDK name of the function that failed.
        function: &'static str,
        /// The TeamSpeak error code (`public_errors.h`).
        code: u32,
        /// The error text TeamSpeak gives for `code`, or a fallback when it gives none.
        message: String,
    },
    /// A text argument holds a NUL byte, so it cannot become the C string TeamSpeak reads.
    #[error("the text passed to TeamSpeak `{function}` holds a NUL byte at byte {position}")]
    InteriorNul {
        /// The SDK name of the function the text was meant for.
        function: &'static str,
        /// The byte offset of the first NUL in the text.
        position: usize,
    },
    /// TeamSpeak reported success but left the result pointer null.
    #[error("TeamSpeak `{function}` reported success but returned no result")]
    NullResult {
        /// The SDK name of the function that returned nothing.
        function: &'static str,
    },
    /// TeamSpeak wrote an empty path into the buffer.
    #[error("TeamSpeak `{function}` returned an empty path")]
    EmptyPath {
        /// The SDK name of the path function.
        function: &'static str,
    },
    /// The call needs the plugin ID, but `ts3plugin_registerPluginID` has not delivered it yet.
    #[error("TeamSpeak `{function}` needs the plugin ID, but none is registered yet")]
    PluginIdNotRegistered {
        /// The SDK name of the function that needs the ID.
        function: &'static str,
    },
    /// TeamSpeak has not called `ts3plugin_setFunctionPointers`, so there is no table to call.
    #[error(
        "the TeamSpeak function table is not installed: ts3plugin_setFunctionPointers was not called"
    )]
    FunctionsNotInstalled,
}

/// Every failure of `tfar_ts3_host` that is not a single TeamSpeak call.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A TeamSpeak call failed.
    #[error(transparent)]
    TeamSpeak(#[from] Ts3Error),
    /// Another `tracing` subscriber is already the process-wide default.
    #[error(
        "another tracing subscriber is already the global default; the TeamSpeak log layer cannot be installed"
    )]
    SubscriberAlreadySet,
    /// A logger thread already forwards this queue; drop its [`LoggingGuard`](crate::LoggingGuard)
    /// first.
    #[error(
        "the TeamSpeak log forwarder is already running; drop its guard before installing again"
    )]
    LoggingAlreadyInstalled,
    /// The operating system refused to start the logger thread.
    #[error("the TeamSpeak logger thread could not start: {reason}")]
    LoggerThreadSpawn {
        /// The operating system's reason.
        reason: String,
    },
}

/// The result of a fallible `tfar_ts3_host` operation that is not a single TeamSpeak call.
pub type Result<T> = core::result::Result<T, Error>;
