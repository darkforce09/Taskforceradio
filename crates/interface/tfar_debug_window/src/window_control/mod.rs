//! The handle's state machine: one UI thread, windows opened and closed on request, the snapshot
//! shared lock-free, and the backend seam that keeps it testable without a display.

mod debug_window_handle;
mod window_backend;
mod window_state;

pub use debug_window_handle::{DebugWindow, DebugWindowHandle, SHUTDOWN_TIMEOUT};
pub(crate) use window_backend::WindowBackend;
pub(crate) use window_state::{Waker, WindowSession};
