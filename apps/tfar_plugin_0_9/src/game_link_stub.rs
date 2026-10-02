//! The game link handler of the stub plugin: it answers the ping and refuses every other command.

use tfar_game_link_server::prelude::GameLinkHandler;

#[cfg(test)]
#[path = "tests/game_link_stub.rs"]
mod tests;

/// The sync call the game makes to test that the plugin is running
/// (`fn_isTeamSpeakPluginEnabled.sqf`).
pub const PING_REQUEST: &[u8] = b"TS_INFO\tPING";

/// The answer to [`PING_REQUEST`].
pub const PING_REPLY: &[u8] = b"PONG";

/// The 0.9 plugin's answer to a command it does not know.
pub const UNKNOWN_COMMAND_REPLY: &[u8] = b"FAIL";

/// Answers [`PING_REQUEST`] with [`PING_REPLY`], every other sync call with
/// [`UNKNOWN_COMMAND_REPLY`], and ignores async messages and focus changes; the 0.9 command
/// handling replaces it.
#[derive(Debug, Default, Clone, Copy)]
pub struct StubGameLinkHandler;

impl GameLinkHandler for StubGameLinkHandler {
    fn sync_request(&self, text: &[u8]) -> Vec<u8> {
        if text == PING_REQUEST {
            PING_REPLY.to_vec()
        } else {
            UNKNOWN_COMMAND_REPLY.to_vec()
        }
    }

    fn async_message(&self, _text: &[u8]) {}

    fn focus_changed(&self, _focused: bool) {}

    fn session_opened(&self) {
        tracing::info!("game link: the Arma extension connected");
    }

    fn session_closed(&self) {
        tracing::info!("game link: the Arma extension disconnected");
    }
}
