//! The identifier newtypes TeamSpeak hands out: server connection, client and channel.
//!
//! Each wraps the SDK's integer type unchanged (`uint64` or `anyID`), so a value crosses the
//! boundary in both directions without conversion and a client ID can never be passed where a
//! channel ID belongs.

/// A server connection handler: one tab of the TeamSpeak client (`uint64` in the SDK).
///
/// `0` is TeamSpeak's "no connection".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ServerConnectionHandlerId(pub u64);

/// A client on a server (`anyID`, a 16-bit integer, in the SDK); `0` terminates TeamSpeak's lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ClientId(pub u16);

/// A channel on a server (`uint64` in the SDK); `0` is the server root and terminates lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ChannelId(pub u64);
