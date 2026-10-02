//! The names a caller of `tfar_ts3_mock` imports with `use tfar_ts3_mock::prelude::*;`, with the
//! `tfar_ts3_host` newtypes and values the mock's API speaks in.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
pub use crate::{
    CapturedVoiceOutcome, CommandOutcome, FakeChannel, FakeClient, FakeServer, FakeTeamSpeak,
    HostCall, HostOptions, HostQuery, InfoKind, LeakReport, MOCK_PLUGIN_ID, MockHost,
    PluginLibrary, PluginMetadata, SentCommandTarget, ServerSetup, VoiceBlockOutcome,
    locate_cdylib,
};
pub use tfar_ts3_host::{
    ChannelId, ClientId, ConnectStatus, LogLevel, ServerConnectionHandlerId, Ts3Vector,
};
