//! The seam between the handle's state machine and the code that draws a real window, so the
//! state machine runs headless in tests.

use super::window_state::WindowSession;
use crate::Result;

/// Opens windows on the UI thread, one at a time.
pub(crate) trait WindowBackend: Send + 'static {
    /// Opens one window and blocks until it has closed: either the session says
    /// [`should_close`](WindowSession::should_close) or the user closed it. A backend registers a
    /// waker with [`WindowSession::set_waker`] so a request reaches it without polling.
    ///
    /// # Errors
    ///
    /// [`Error::Window`](crate::Error::Window) when the window cannot open.
    fn run_window(&mut self, session: &WindowSession<'_>) -> Result<()>;
}
