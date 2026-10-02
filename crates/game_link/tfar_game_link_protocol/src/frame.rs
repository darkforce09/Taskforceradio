//! The six frame kinds of the game link and the `HELLO` handshake values.
//!
//! **Role:** the in-memory form of every frame, the kind codes of the wire table and the check of a
//! peer's `HELLO` against the local flavour.
//!
//! **Position:** tier 0 of `crates/game_link`; built and matched by `tfar_game_link_server` and
//! `tfar_game_link_client`, turned into bytes by the frame codec.
//!
//! **Signals and state:** none; plain values.
//!
//! **Invariants:** the kind codes are the wire table's (`HELLO` 0x01 to `STATUS` 0x06); text fields
//! hold the bytes exactly as Arma passed them or the plugin built them, never re-encoded.

use serde::{Deserialize, Serialize};

use crate::flavour::Flavour;
use crate::protocol_constants::{MAGIC, PROTOCOL_VERSION};

#[cfg(test)]
#[path = "tests/frame.rs"]
mod tests;

/// The kind byte that follows a frame's length prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FrameKind {
    /// `HELLO`, sent by both sides to open a session.
    Hello = 0x01,
    /// `SYNC_REQ`, a call the extension waits on.
    SyncRequest = 0x02,
    /// `SYNC_RESP`, the plugin's answer to one `SYNC_REQ`.
    SyncResponse = 0x03,
    /// `ASYNC`, a 1.0 `~` call the extension does not wait on.
    Async = 0x04,
    /// `FOCUS`, whether Arma's window is in front (1.0).
    Focus = 0x05,
    /// `STATUS`, the plugin's heartbeat carrying needcfg.
    Status = 0x06,
}

impl FrameKind {
    /// The kind's byte on the wire.
    pub const fn code(self) -> u8 {
        self as u8
    }

    /// The kind a wire byte names, or `None` for a byte outside the table.
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            0x01 => Some(Self::Hello),
            0x02 => Some(Self::SyncRequest),
            0x03 => Some(Self::SyncResponse),
            0x04 => Some(Self::Async),
            0x05 => Some(Self::Focus),
            0x06 => Some(Self::Status),
            _ => None,
        }
    }

    /// The kind's name in the protocol table, for log lines.
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Hello => "HELLO",
            Self::SyncRequest => "SYNC_REQ",
            Self::SyncResponse => "SYNC_RESP",
            Self::Async => "ASYNC",
            Self::Focus => "FOCUS",
            Self::Status => "STATUS",
        }
    }
}

impl core::fmt::Display for FrameKind {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.wire_name())
    }
}

/// The payload of a `HELLO`: the magic, the protocol number and the flavour byte, in wire order.
///
/// The flavour stays a raw byte so a peer's unknown flavour decodes and can be logged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hello {
    /// The four magic bytes; [`MAGIC`] from a real peer.
    pub magic: [u8; 4],
    /// The protocol number; [`PROTOCOL_VERSION`] from a compatible peer.
    pub protocol: u16,
    /// The flavour byte, [`Flavour::wire_value`] of the sender's mod version.
    pub flavour: u8,
}

impl Hello {
    /// The `HELLO` an endpoint of `flavour` sends.
    pub const fn new(flavour: Flavour) -> Self {
        Self {
            magic: MAGIC,
            protocol: PROTOCOL_VERSION,
            flavour: flavour.wire_value(),
        }
    }

    /// The first way this `HELLO` differs from what an endpoint of `expected` accepts, checked in
    /// the order magic, protocol, flavour; `None` when it is accepted.
    pub const fn mismatch_with(&self, expected: Flavour) -> Option<HelloMismatch> {
        if !matches!(self.magic, MAGIC) {
            return Some(HelloMismatch::Magic { found: self.magic });
        }
        if self.protocol != PROTOCOL_VERSION {
            return Some(HelloMismatch::Protocol {
                found: self.protocol,
            });
        }
        if self.flavour != expected.wire_value() {
            return Some(HelloMismatch::Flavour {
                found: self.flavour,
                expected,
            });
        }
        None
    }
}

/// Why a peer's `HELLO` is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HelloMismatch {
    /// The magic is not [`MAGIC`].
    Magic {
        /// The magic the peer sent.
        found: [u8; 4],
    },
    /// The protocol number is not [`PROTOCOL_VERSION`].
    Protocol {
        /// The protocol number the peer sent.
        found: u16,
    },
    /// The flavour is not the receiver's own.
    Flavour {
        /// The flavour byte the peer sent.
        found: u8,
        /// The receiver's flavour.
        expected: Flavour,
    },
}

impl core::fmt::Display for HelloMismatch {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Magic { found } => write!(
                formatter,
                "magic {found:02x?} is not {MAGIC:02x?} (\"TFAR\")"
            ),
            Self::Protocol { found } => {
                write!(formatter, "protocol {found} is not {PROTOCOL_VERSION}")
            }
            Self::Flavour { found, expected } => write!(
                formatter,
                "flavour {found} is not {} ({expected})",
                expected.wire_value()
            ),
        }
    }
}

/// One game link frame, with its kind's payload fields.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Frame {
    /// `HELLO`: opens a session; the plugin answers with its own after checking the extension's.
    Hello(Hello),
    /// `SYNC_REQ`: a call the extension waits on.
    SyncRequest {
        /// Counts up from 1 per session.
        id: u32,
        /// The call text exactly as Arma passed it, without the NUL.
        text: Vec<u8>,
    },
    /// `SYNC_RESP`: the plugin's answer to the `SYNC_REQ` with the same id.
    SyncResponse {
        /// The id of the request answered.
        id: u32,
        /// The reply exactly as the plugin built it.
        text: Vec<u8>,
    },
    /// `ASYNC`: a 1.0 call ending in `~`, sent with the `~` removed.
    Async {
        /// The call text without its trailing `~`.
        text: Vec<u8>,
    },
    /// `FOCUS`: whether Arma's window is in front (1.0).
    Focus {
        /// True when Arma's window is the foreground window.
        arma_focused: bool,
    },
    /// `STATUS`: the plugin's heartbeat; needcfg is always false for 0.9.
    Status {
        /// True while the plugin waits for the 1.0 SETCFG settings.
        needcfg: bool,
    },
}

impl Frame {
    /// The `HELLO` frame an endpoint of `flavour` sends.
    pub const fn hello(flavour: Flavour) -> Self {
        Self::Hello(Hello::new(flavour))
    }

    /// The frame's kind.
    pub const fn kind(&self) -> FrameKind {
        match self {
            Self::Hello(_) => FrameKind::Hello,
            Self::SyncRequest { .. } => FrameKind::SyncRequest,
            Self::SyncResponse { .. } => FrameKind::SyncResponse,
            Self::Async { .. } => FrameKind::Async,
            Self::Focus { .. } => FrameKind::Focus,
            Self::Status { .. } => FrameKind::Status,
        }
    }
}
