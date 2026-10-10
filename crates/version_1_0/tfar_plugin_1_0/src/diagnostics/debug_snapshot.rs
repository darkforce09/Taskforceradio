//! The debug window's content: a snapshot of the plugin, the settings, the clients and the recent
//! game messages, published by the service thread every 500 ms while the window is open.

use tfar_debug_window::prelude::{DebugSection, DebugSnapshot, DebugTable};

use crate::diagnostics::reports::{diagnostics_report, setting_rows};
use crate::runtime::PluginRuntime;

/// How many recent game messages the window shows.
pub const SHOWN_GAME_MESSAGES: usize = 40;

/// The snapshot of `runtime` the window shows.
pub fn debug_snapshot(runtime: &PluginRuntime) -> DebugSnapshot {
    let state_rows: Vec<(String, String)> = diagnostics_report(runtime)
        .lines()
        .filter_map(|line| line.split_once(": "))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
    let mut clients = DebugTable::new(["nickname", "id", "alive", "talking", "transmitting"]);
    if let Some((_, state)) = runtime.current_server() {
        let now = std::time::Instant::now();
        let state = state.lock();
        let frame = state.session.game.current_data_frame;
        for client in state.session.clients.iter() {
            clients.push_row([
                client.nickname().to_owned(),
                client.client_id().0.to_string(),
                client.is_alive(now, frame).to_string(),
                client.is_talking_now.to_string(),
                client.is_transmitting().to_string(),
            ]);
        }
    }
    let mut messages = DebugTable::new(["game message"]);
    {
        let ring = runtime.game_messages.lock();
        let skip = ring.len().saturating_sub(SHOWN_GAME_MESSAGES);
        for message in ring.iter().skip(skip) {
            messages.push_row([message.replace('\t', " | ")]);
        }
    }
    DebugSnapshot::new()
        .with_section(DebugSection::key_values("Plugin", state_rows))
        .with_section(DebugSection::key_values("Settings", setting_rows(runtime)))
        .with_section(DebugSection::table("Clients", clients))
        .with_section(DebugSection::table("Game messages", messages))
}

/// Publishes a snapshot when the debug window is open.
pub fn publish_debug_snapshot(runtime: &PluginRuntime) {
    let is_open = runtime
        .debug_window
        .lock()
        .as_ref()
        .is_some_and(tfar_debug_window::prelude::DebugWindowHandle::is_open);
    if !is_open {
        return;
    }
    let snapshot = debug_snapshot(runtime);
    if let Some(window) = runtime.debug_window.lock().as_ref() {
        window.publish(snapshot);
    }
}
