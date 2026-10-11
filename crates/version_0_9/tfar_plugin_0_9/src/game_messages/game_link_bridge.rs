//! `GameLinkBridge`: the plugin's `GameLinkHandler`, from the game link server to `PluginCore`.

use std::sync::Arc;
use std::time::Instant;

use tfar_game_link::server::prelude::GameLinkHandler;

use crate::plugin_core::PluginCore;

/// Hands every sync request of the game link to [`PluginCore::handle_game_message`] and records
/// whether an Arma extension is connected. Runs on the game link's session thread.
#[derive(Debug, Clone)]
pub struct GameLinkBridge {
    core: Arc<PluginCore>,
}

impl GameLinkBridge {
    /// A bridge into `core`.
    pub fn new(core: Arc<PluginCore>) -> Self {
        Self { core }
    }
}

impl GameLinkHandler for GameLinkBridge {
    fn sync_request(&self, text: &[u8]) -> Vec<u8> {
        let text = String::from_utf8_lossy(text);
        self.core
            .handle_game_message(&text, Instant::now())
            .into_bytes()
    }

    fn async_message(&self, text: &[u8]) {
        // The 0.9 extension sends everything sync; an async frame is a 1.0 extension's.
        tracing::debug!(
            bytes = text.len(),
            "an async game message reached the 0.9 plugin"
        );
    }

    fn focus_changed(&self, _focused: bool) {}

    fn session_opened(&self) {
        tracing::info!("game link: the Arma extension connected");
        self.core.set_link_connected(true);
    }

    fn session_closed(&self) {
        tracing::info!("game link: the Arma extension disconnected");
        self.core.set_link_connected(false);
        let server = self.core.team_speak().current_server();
        self.core.publish_status_metadata(server);
    }
}
