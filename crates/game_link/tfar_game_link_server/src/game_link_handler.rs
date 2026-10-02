//! The callbacks through which the plugin answers and observes the game link.

/// What the plugin does with the game link's traffic; the server owns no command logic.
///
/// Callbacks run on the server's threads: [`sync_request`](Self::sync_request),
/// [`focus_changed`](Self::focus_changed), [`session_opened`](Self::session_opened) and
/// [`session_closed`](Self::session_closed) on a session thread, and
/// [`async_message`](Self::async_message) on the async worker. A callback may call
/// [`GameLinkServer::set_needcfg`](crate::GameLinkServer::set_needcfg) and the read-only
/// accessors, but never [`GameLinkServer::shutdown`](crate::GameLinkServer::shutdown), which waits
/// for these threads.
pub trait GameLinkHandler: Send + Sync {
    /// Answers one `SYNC_REQ`: `text` is the call exactly as Arma passed it, the return value the
    /// reply exactly as the extension hands it back. Runs on the session thread, which reads no
    /// further frame until it returns.
    fn sync_request(&self, text: &[u8]) -> Vec<u8>;

    /// Runs one `ASYNC` message (the call without its `~`) on the async worker, strictly in arrival
    /// order; a `MISSIONEND` drops the messages queued before it, then is delivered itself.
    fn async_message(&self, text: &[u8]);

    /// Arma's window came to the front (`true`) or left it (`false`), from a `FOCUS` frame.
    fn focus_changed(&self, focused: bool);

    /// A session passed the handshake; the link is connected. Before the first
    /// [`focus_changed`](Self::focus_changed) of a session, Arma counts as not in front.
    fn session_opened(&self);

    /// The session ended: the peer closed it, a frame failed to decode, a write failed, a newer
    /// session replaced it, or the server shut down. Always follows its
    /// [`session_opened`](Self::session_opened), before the next session's.
    fn session_closed(&self);
}
