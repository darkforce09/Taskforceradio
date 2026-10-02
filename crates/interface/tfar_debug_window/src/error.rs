//! The error type of `tfar_debug_window`.

use std::time::Duration;

/// Every failure `tfar_debug_window` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// `DISPLAY` is unset or empty, so no X11 window can open.
    #[error("no X11 display: DISPLAY is not set, so the debug window cannot open")]
    NoDisplay,
    /// The process already has its UI thread; winit allows one event loop per process.
    #[error("the debug window's UI thread is already running in this process")]
    AlreadySpawned,
    /// The operating system refused to start the UI thread.
    #[error("the debug window's UI thread could not start: {source}")]
    ThreadSpawn {
        /// What the operating system reported.
        #[source]
        source: std::io::Error,
    },
    /// The UI thread did not end in time after a shutdown request.
    #[error("the debug window's UI thread did not end within {waited:?}")]
    ShutdownTimedOut {
        /// How long the shutdown waited.
        waited: Duration,
    },
    /// The UI thread ended by panicking.
    #[error("the debug window's UI thread panicked")]
    UiThreadPanicked,
    /// A window could not open: no X11 connection, no OpenGL context, or a second event loop.
    #[error("the debug window could not open: {reason}")]
    Window {
        /// What eframe reported.
        reason: String,
    },
}

/// The result of a fallible `tfar_debug_window` operation.
pub type Result<T> = core::result::Result<T, Error>;
