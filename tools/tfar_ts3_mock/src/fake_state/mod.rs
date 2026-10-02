//! The fake TeamSpeak state, its allocation ledger and the session that owns it.

pub(crate) mod allocation_ledger;
pub(crate) mod fake_teamspeak;
pub(crate) mod host_session;

pub use allocation_ledger::{LeakReport, UnfreedAllocation};
pub use fake_teamspeak::{FakeChannel, FakeClient, FakeServer, FakeTeamSpeak};
