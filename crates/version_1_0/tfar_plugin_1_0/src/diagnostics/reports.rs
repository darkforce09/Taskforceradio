//! The text reports of `/tfar diag`, `/tfar pos` and the diagnostics dump: the plugin's state,
//! the clients with their positions, the settings and the recent game messages.
//!
//! **Role:** `doDiagReport` and `doTypeDiagReport("pos")` of `reference/tfar_1_0/ts/src/
//! task_force_radio.cpp` as plain text, built from the runtime under short locks.
//!
//! **Position:** used by the chat commands, the dump and the debug window snapshot.
//!
//! **Signals and state:** reads the runtime; holds one server lock at a time and calls no
//! TeamSpeak function while it does.
//!
//! **Invariants:** a report never changes plugin state.

use std::fmt::Write as _;
use std::time::Instant;

use tfar_plugin_settings_1_0::prelude::{SettingKey, SettingValue};

use crate::runtime::PluginRuntime;

/// Every setting the 1.0 plugin knows, in the original's table order.
pub const SETTING_KEYS: [SettingKey; 20] = [
    SettingKey::FullDuplex,
    SettingKey::AddonVersion,
    SettingKey::SeriousChannelName,
    SettingKey::SeriousChannelPassword,
    SettingKey::IntercomVolume,
    SettingKey::IntercomEnabled,
    SettingKey::PluginTimeout,
    SettingKey::HeadsetLowered,
    SettingKey::SpectatorNotHearEnemies,
    SettingKey::SpectatorCanHearFriendlies,
    SettingKey::TangentReleaseDelay,
    SettingKey::MoveWhileTabbedOut,
    SettingKey::IntercomDucking,
    SettingKey::MinimumPluginVersion,
    SettingKey::ObjectInterceptionStrength,
    SettingKey::VoiceCone,
    SettingKey::AllowDebugging,
    SettingKey::NoAutomoveSpectator,
    SettingKey::DisableAutomaticMute,
    SettingKey::MuteSpectators,
];

/// A setting's value as text.
pub fn setting_text(value: &SettingValue) -> String {
    match value {
        SettingValue::Bool(flag) => flag.to_string(),
        SettingValue::Number(number) => number.to_string(),
        SettingValue::Text(text) => text.clone(),
    }
}

/// Every setting as `(name, value)`.
pub fn setting_rows(runtime: &PluginRuntime) -> Vec<(String, String)> {
    let settings = runtime.settings();
    SETTING_KEYS
        .iter()
        .map(|key| {
            (
                key.wire_name().to_owned(),
                setting_text(&settings.value(*key)),
            )
        })
        .collect()
}

/// The plugin's state: game, link, server, serious mode, clients, towers, sounds.
pub fn diagnostics_report(runtime: &PluginRuntime) -> String {
    let mut report = String::new();
    let _ = writeln!(report, "in game: {}", runtime.is_in_game());
    let _ = writeln!(report, "dormant: {}", runtime.is_dormant());
    let _ = writeln!(report, "game link session: {}", runtime.is_link_connected());
    let _ = writeln!(report, "Arma in front: {}", runtime.is_arma_focused());
    let _ = writeln!(
        report,
        "HRTF: {}",
        if runtime.sounds.hrir_library().is_some() {
            "on"
        } else {
            "off"
        }
    );
    let _ = writeln!(
        report,
        "radio sounds playing: {}",
        runtime.sounds.mixer.active_playback_count()
    );
    if let Some((server, state)) = runtime.current_server() {
        let state = state.lock();
        let game = &state.session.game;
        let _ = writeln!(report, "server: {}", server.0);
        let _ = writeln!(report, "serious mode: {}", state.is_serious_mode);
        let _ = writeln!(report, "alive: {}", game.is_alive);
        let _ = writeln!(report, "data frame: {:?}", game.current_data_frame.number());
        let _ = writeln!(report, "tangent pressed: {}", game.is_tangent_pressed);
        let _ = writeln!(report, "clients: {}", state.session.clients.len());
        let _ = writeln!(report, "muted by TFAR: {}", state.mutes.muted().count());
        let _ = writeln!(report, "radio towers: {}", state.antennas.len());
        let _ = writeln!(report, "speakers: {}", game.speakers.len());
    }
    report
}

/// One line per known client: nickname, id, position, alive, talking, transmitting.
pub fn positions_report(runtime: &PluginRuntime) -> String {
    let mut report = String::new();
    let Some((_, state)) = runtime.current_server() else {
        return report;
    };
    let now = Instant::now();
    let state = state.lock();
    let frame = state.session.game.current_data_frame;
    for client in state.session.clients.iter() {
        let position = client.client_position(now);
        let _ = writeln!(
            report,
            "{} ({}): [{:.1}, {:.1}, {:.1}] alive {} talking {} transmitting {}",
            client.nickname(),
            client.client_id().0,
            position.x(),
            position.y(),
            position.z(),
            client.is_alive(now, frame),
            client.is_talking_now,
            client.is_transmitting(),
        );
    }
    report
}

/// The settings as `name = value` lines.
pub fn settings_report(runtime: &PluginRuntime) -> String {
    let mut report = String::new();
    for (name, value) in setting_rows(runtime) {
        let _ = writeln!(report, "{name} = {value}");
    }
    report
}

/// The recent game messages, oldest first, one per line.
pub fn game_messages_report(runtime: &PluginRuntime) -> String {
    let mut report = String::new();
    for message in runtime.game_messages.lock().iter() {
        let _ = writeln!(report, "{}", message.replace('\t', " | "));
    }
    report
}
