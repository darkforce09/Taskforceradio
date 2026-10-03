//! [`LinkHandler`]: the plugin's answer to the game link: sync calls answered, async messages
//! applied in order, focus and session changes recorded.
//!
//! **Role:** the `GameLinkHandler` the game link server calls; it logs each game message, counts
//! it for the game timeout (running a due connect transition first), and hands it to
//! [`sync_reply`] or [`apply_async_message`].
//!
//! **Position:** created in `ts3plugin_init` with a weak handle on the runtime; the link server's
//! session thread calls the sync and focus methods, its async worker the async one.
//!
//! **Signals and state:** a `Weak<PluginRuntime>`, so the server's threads never keep the runtime
//! alive past shutdown; a call after the runtime is gone answers the empty text.
//!
//! **Invariants:** `HELLO` and `FOCUS` never count as game messages; `MISSIONEND` forgets the
//! game's messages instead of counting.

use std::sync::Weak;
use std::time::Instant;

use tfar_game_commands_1_0::prelude::{GameCommand, parse};
use tfar_game_link_server::prelude::GameLinkHandler;

use crate::game_link::async_commands::apply_async_message;
use crate::game_link::sync_replies::sync_reply;
use crate::runtime::PluginRuntime;

/// The command name of a position update, which the sync path logs before it queues it.
const POSITION_PREFIX: &str = "POS\t";

/// The game link handler of the running plugin; see the module header.
#[derive(Debug)]
pub struct LinkHandler {
    runtime: Weak<PluginRuntime>,
}

impl LinkHandler {
    /// A handler for `runtime`.
    pub const fn new(runtime: Weak<PluginRuntime>) -> Self {
        Self { runtime }
    }
}

impl GameLinkHandler for LinkHandler {
    fn sync_request(&self, text: &[u8]) -> Vec<u8> {
        profiling::scope!("game_link_sync_request");
        let Some(runtime) = self.runtime.upgrade() else {
            return Vec::new();
        };
        let text = String::from_utf8_lossy(text);
        runtime.game_messages.lock().push(&text);
        runtime.record_game_message(Instant::now());
        sync_reply(&runtime, &text).into_bytes()
    }

    fn async_message(&self, text: &[u8]) {
        profiling::scope!("game_link_async_message");
        let Some(runtime) = self.runtime.upgrade() else {
            return;
        };
        let text = String::from_utf8_lossy(text);
        // A `POS` here is the one the sync path queued and logged already, as the original skips
        // logging it twice (`CommandProcessor.cpp:225`).
        if !text.starts_with(POSITION_PREFIX) {
            runtime.game_messages.lock().push(&text);
        }
        match parse(&text) {
            Ok(GameCommand::MissionEnd) => runtime.end_mission(Instant::now()),
            Ok(command) => {
                runtime.record_game_message(Instant::now());
                apply_async_message(&runtime, &text, command);
            }
            Err(error) => {
                runtime.record_game_message(Instant::now());
                tracing::warn!(%error, "a game message was rejected");
            }
        }
    }

    fn focus_changed(&self, focused: bool) {
        if let Some(runtime) = self.runtime.upgrade() {
            runtime.set_arma_focused(focused);
        }
    }

    fn session_opened(&self) {
        tracing::info!("game link: the Arma extension connected");
    }

    fn session_closed(&self) {
        tracing::info!("game link: the Arma extension disconnected");
        if let Some(runtime) = self.runtime.upgrade() {
            runtime.set_arma_focused(false);
        }
    }
}
