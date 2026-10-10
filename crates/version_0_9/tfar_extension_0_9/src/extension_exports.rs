//! The `RVExtension` and `RVExtensionVersion` exports of the 0.9 DLL and the process-wide game link
//! client they answer through.

use std::sync::OnceLock;

use tfar_arma_extension_abi::arma_command_line_contains;
use tfar_game_link_client::prelude::{ClientConfig, Flavour, GameLinkClient, port};

use crate::call_handling::answer_call;

#[cfg(test)]
#[path = "tests/extension_exports.rs"]
mod tests;

/// The mod version this extension speaks for.
pub const FLAVOUR: Flavour = Flavour::V0_9;

/// The Arma launch switch that selects the plugin's debug port, as the original
/// `isDebugArmaInstance` tests it.
pub const DEBUG_SWITCH: &str = "-tfdebug";

/// What `RVExtensionVersion` writes: the crate name and its version.
pub const EXTENSION_VERSION: &str = concat!("tfar_extension_0_9 ", env!("CARGO_PKG_VERSION"));

/// The one game link client of the DLL, created on the first call.
static GAME_LINK_CLIENT: OnceLock<GameLinkClient> = OnceLock::new();

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

/// The `RVExtension` handler: one call, one reply.
fn answer_game_call(input: &str) -> String {
    answer_call(game_link_client(), input)
}

tfar_arma_extension_abi::arma_extension!(handler: answer_game_call, version: EXTENSION_VERSION);
