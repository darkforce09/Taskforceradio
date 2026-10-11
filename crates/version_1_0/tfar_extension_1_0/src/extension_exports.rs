//! The `RVExtension` and `RVExtensionVersion` exports of the 1.0 DLL, the process-wide game link
//! client they answer through and the focus watch every call consults.

use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::Instant;

use tfar_arma_extension_abi::{arma_command_line_contains, foreground_window_is_arma};
use tfar_game_link::client::prelude::{ClientConfig, Flavour, GameLinkClient, port};

use crate::call_handling::answer_call;
use crate::focus_watch::FocusWatch;

#[cfg(test)]
#[path = "tests/extension_exports.rs"]
mod tests;

/// The mod version this extension speaks for.
pub const FLAVOUR: Flavour = Flavour::V1_0;

/// The Arma launch switch that selects the plugin's debug port, as the original
/// `isDebugArmaInstance` tests it.
pub const DEBUG_SWITCH: &str = "-tfdebug";

/// What `RVExtensionVersion` writes: the crate name and its version.
pub const EXTENSION_VERSION: &str = concat!("tfar_extension_1_0 ", env!("CARGO_PKG_VERSION"));

/// The one game link client of the DLL, created on the first call.
static GAME_LINK_CLIENT: OnceLock<GameLinkClient> = OnceLock::new();

/// When the foreground window was read last and the focus value sent last.
static FOCUS_WATCH: Mutex<FocusWatch> = Mutex::new(FocusWatch::new());

/// The client settings for the port `debug` selects, every timeout at the spec's value.
pub const fn client_config(debug: bool) -> ClientConfig {
    ClientConfig::new(FLAVOUR, port(FLAVOUR, debug))
}

/// The process-wide client; the first call reads Arma's command line for [`DEBUG_SWITCH`].
fn game_link_client() -> &'static GameLinkClient {
    GAME_LINK_CLIENT.get_or_init(|| {
        GameLinkClient::new(client_config(arma_command_line_contains(DEBUG_SWITCH)))
    })
}

/// The `RVExtension` handler: one call, one reply, with the focus check on the way.
fn answer_game_call(input: &str) -> String {
    // The watch holds two plain values that every poll leaves consistent, so a poisoned lock is
    // still usable (and with `panic = "abort"` it never poisons).
    let mut focus = FOCUS_WATCH.lock().unwrap_or_else(PoisonError::into_inner);
    answer_call(
        game_link_client(),
        &mut focus,
        Instant::now(),
        foreground_window_is_arma,
        input,
    )
}

tfar_arma_extension_abi::arma_extension!(handler: answer_game_call, version: EXTENSION_VERSION);
