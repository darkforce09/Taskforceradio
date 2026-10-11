//! The outcome of one sync request.

/// What came of one [`GameLinkClient::sync_request`](crate::client::GameLinkClient::sync_request).
///
/// The extension apps map each outcome to the reply their original returned; this crate holds no
/// game-facing string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncOutcome {
    /// The plugin's `SYNC_RESP` text for this request, byte for byte.
    Reply(Vec<u8>),
    /// No session was up and none could be opened: the connect failed or was refused, the handshake
    /// failed, or the backoff after a failure or a drop was still running. Nothing was sent.
    NoSession,
    /// The plugin closed the connection (or a write to it failed) while the call was in flight.
    ClosedDuringCall,
    /// No matching `SYNC_RESP` arrived within the sync timeout (or the write did not finish within
    /// the write timeout). A late response is discarded by the next request.
    TimedOut,
}
