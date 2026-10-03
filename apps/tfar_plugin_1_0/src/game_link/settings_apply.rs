//! `SETCFG`: one plugin setting from the game, applied through `tfar_plugin_settings_1_0`, with
//! the needcfg push, the serious-mode check and the one-time version warning.
//!
//! **Role:** the `SETCFG` case of `reference/tfar_1_0/ts/src/CommandProcessor.cpp:388-414` and
//! the `configValueSet` handler of `task_force_radio.cpp:73-100`.
//!
//! **Position:** called by the async message path on the link's async worker.
//!
//! **Signals and state:** replaces the runtime's settings snapshot; reads the current server.
//!
//! **Invariants:** an unknown key leaves the settings unchanged and tells the user in the chat
//! tab with the original's text; every accepted set publishes needcfg and re-checks serious mode.

use std::sync::atomic::Ordering;

use tfar_game_commands_1_0::prelude::ConfigAssignment;
use tfar_plugin_settings_1_0::prelude::SettingKey;

use crate::runtime::PluginRuntime;

/// The plugin build number `minimumPluginVersion` is compared with: the original's value for a
/// build that is not a numbered CI build.
pub const PLUGIN_BUILD_NUMBER: f32 = 5000.0;

/// The warning when the game asks for a newer plugin (`task_force_radio.cpp:94-97`).
pub const MINIMUM_VERSION_WARNING: &str = "Arma requires a newer TFAR Plugin version than the one you have installed. Please update your Plugin.";

/// Applies one `SETCFG`.
pub fn apply_set_config(runtime: &PluginRuntime, assignment: &ConfigAssignment) {
    let result = runtime.update_settings(|settings| {
        settings.set(&assignment.key, &assignment.value, assignment.type_tag)
    });
    let changed = match result {
        Ok(changed) => changed,
        Err(error @ tfar_plugin_settings_1_0::Error::UnknownKey { .. }) => {
            tracing::warn!(%error, "SETCFG named no setting");
            if let Some(ts) = runtime.act() {
                ts.print_to_current_tab(&error.to_string());
            }
            return;
        }
        Err(error) => {
            tracing::warn!(%error, "SETCFG was refused");
            return;
        }
    };
    let settings = runtime.settings();
    runtime.set_needcfg(settings.needs_refresh());
    let server = runtime.ts().current_server();
    if changed.key == SettingKey::SeriousChannelName {
        runtime.refresh_channels(server);
        runtime.move_to_serious_channel(server, false);
    }
    runtime.check_serious_mode(server);
    if changed.key == SettingKey::MinimumPluginVersion
        && settings.minimum_plugin_version() > PLUGIN_BUILD_NUMBER
        && !runtime
            .has_warned_minimum_version
            .swap(true, Ordering::AcqRel)
        && let Some(ts) = runtime.act()
    {
        ts.print_to_current_tab(MINIMUM_VERSION_WARNING);
    }
    if let Some((_, state)) = runtime.current_server() {
        runtime.ensure_every_sender_dsp(&mut state.lock(), &settings);
    }
}
