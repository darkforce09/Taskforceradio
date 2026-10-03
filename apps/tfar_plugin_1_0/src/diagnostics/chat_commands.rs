//! `/tfar <command>`: the chat commands of `ts3plugin_processCommand`
//! (`reference/tfar_1_0/ts/src/Teamspeak.cpp:838-925`).
//!
//! `diag`, `pos`, `full…` and `debug` work only with `allowDebugging`; `rstflt` always.
//! `debug` opens the debug window, or says in the chat tab why it cannot (no X11 display).

use tfar_debug_window::prelude::DebugWindow;
use tfar_ts3_host::CommandHandling;

use crate::diagnostics::debug_dump::write_debug_dump;
use crate::diagnostics::reports::{diagnostics_report, positions_report};
use crate::runtime::PluginRuntime;

/// The title of the debug window.
pub const DEBUG_WINDOW_TITLE: &str = "Task Force Arrowhead Radio debug";

/// Runs the chat command `command` (the text after `/tfar `).
pub fn run_chat_command(runtime: &PluginRuntime, command: &str) -> CommandHandling {
    let command = command.trim();
    let allows_debugging = runtime.settings().allow_debugging();
    let ts = runtime.ts();
    match command {
        "diag" if allows_debugging => {
            ts.print_to_current_tab(&format!("diag\n{}", diagnostics_report(runtime)));
        }
        "pos" if allows_debugging => ts.print_to_current_tab(&positions_report(runtime)),
        "debug" if allows_debugging => open_debug_window(runtime),
        "rstflt" => {
            for (_, state) in runtime.servers.all() {
                for client in state.lock().session.clients.iter_mut() {
                    client.effects.reset_radio_effects();
                }
            }
            ts.print_to_current_tab("Task Force Radio: every radio effect was reset.");
        }
        full if allows_debugging && full.starts_with("full") => {
            write_debug_dump(runtime);
            ts.print_to_current_tab("Task Force Radio: the diagnostics dump was written.");
        }
        _ => return CommandHandling::NotHandled,
    }
    CommandHandling::Handled
}

/// Opens the debug window, starting its UI thread on first use; without a display, says so in
/// the chat tab.
fn open_debug_window(runtime: &PluginRuntime) {
    let mut window = runtime.debug_window.lock();
    if window.is_none() {
        match DebugWindow::spawn(DEBUG_WINDOW_TITLE) {
            Ok(handle) => *window = Some(handle),
            Err(error) => {
                drop(window);
                runtime
                    .ts()
                    .print_to_current_tab(&format!("Task Force Radio: {error}"));
                return;
            }
        }
    }
    if let Some(handle) = window.as_ref() {
        handle.show();
    }
}
