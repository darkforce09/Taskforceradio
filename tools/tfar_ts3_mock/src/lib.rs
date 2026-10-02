//! A fake TeamSpeak host that loads a plugin .so and drives it through scripted scenarios.
//!
//! **Role:** the test host for the plugins: [`MockHost`] loads a plugin `.so` with `libloading`,
//! hands it a fake `Ts3Functions` table (a typed `extern "C"` fake for every function TFAR calls,
//! a recording stub in every other slot), plays server, client and audio events into it, logs
//! every call it makes as a [`HostCall`] and reports leaked TeamSpeak memory.
//!
//! **Position:** `tools`, tier 2, depending on `tfar_ts3_host` (the SDK newtypes),
//! `tfar_game_link_protocol` and `tfar_game_link_client`; the plugin apps' integration tests use
//! it with [`locate_cdylib`], and developers run the binary.
//!
//! **Signals and state:** one process-global [`FakeTeamSpeak`] behind a `parking_lot` mutex, which
//! the fakes reach from any plugin thread; one [`MockHost`] at a time owns it.
//!
//! **Invariants:** no plugin code runs while the fake state is locked; every string and list a
//! fake hands out is `malloc`ed and accounted for until `freeMemory`; a scenario's outcome depends
//! only on the scenario and the plugin.

// The crate is the host side of the plugin FFI: it loads libraries, calls their exports and
// implements the C function table.
#![allow(unsafe_code)]

mod crate_role;
mod error;
mod fake_functions;
mod fake_state;
mod host_call;
mod locate_cdylib;
mod mock_host;
mod plugin_library;
pub mod prelude;
mod temporary_folder;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
pub use fake_functions::{FAKED_FUNCTIONS, FakedFunction, SLOT_COUNT, mock_error_text};
pub use fake_state::{
    FakeChannel, FakeClient, FakeServer, FakeTeamSpeak, LeakReport, UnfreedAllocation,
};
pub use host_call::{HostCall, HostQuery, SentCommandTarget};
pub use locate_cdylib::locate_cdylib;
pub use mock_host::{
    CapturedVoiceOutcome, CommandOutcome, HOSTED_API_VERSION, HostOptions, InfoKind,
    MOCK_PLUGIN_ID, MockHost, ServerSetup, VoiceBlockOutcome,
};
pub use plugin_library::{
    ClientMoveExport, InvokedChannelExport, InvokedChannelUpdateExport, InvokedClientMoveExport,
    NumberExport, PluginExports, PluginLibrary, PluginMetadata, TextExport,
};
