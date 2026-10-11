//! The local player's radio tangent from the game (`TANGENT`, `TANGENT_LR`, `TANGENT_DD`,
//! `RELEASE_ALL_TANGENTS`): the state change through the session, then the local start and end
//! sounds, push-to-talk and the broadcast to the channel.
//!
//! **Role:** the TeamSpeak half of the `TANGENT` and `RELEASE_ALL_TANGENTS` cases of
//! `reference/tfar_1_0/ts/src/CommandProcessor.cpp:293-387`. The state half (the tangent flag, the
//! own transmit state, the half-duplex radio, the sound and the broadcast to send) is
//! `SessionState::apply_local_tangent` and `apply_local_release_all_tangents` of
//! `tfar_plugin_logic_1_0::client_state`.
//!
//! **Position:** called by the async message path on the link's async worker.
//!
//! **Signals and state:** the current server's session under its lock, only for the apply; the
//! push-to-talk state of the runtime after it is released.
//!
//! **Invariants:** only a change of the tangent state acts; `RELEASE_ALL_TANGENTS` also clears the
//! own tangent and the half-duplex radio, so the next press is a change again.

use tfar_plugin_logic_1_0::game_commands::prelude::TangentPress;
use tfar_plugin_logic_1_0::plugin_commands::prelude::{PluginCommand, encode};

use crate::runtime::PluginRuntime;

/// A local tangent press or release.
pub fn apply_local_tangent(runtime: &PluginRuntime, press: &TangentPress) {
    let Some((server, state)) = runtime.current_server() else {
        return;
    };
    let settings = runtime.settings();
    let Some(applied) = state.lock().session.apply_local_tangent(press, &settings) else {
        return;
    };
    if let Some(sound) = &applied.sound {
        runtime
            .sounds
            .play_local(sound.sound, sound.gain, sound.stereo_mode);
    }
    let Some(ts) = runtime.act() else {
        return;
    };
    let broadcast = encode(&PluginCommand::Tangent(applied.broadcast));
    if applied.is_pressed {
        runtime.push_to_talk.press(ts, server, &broadcast);
    } else {
        let delay = runtime
            .push_to_talk_delay()
            .saturating_add(settings.tangent_release_delay());
        runtime.push_to_talk.release(ts, server, broadcast, delay);
    }
}

/// `RELEASE_ALL_TANGENTS`: every own radio is released at once and the channel told.
pub fn release_all_tangents(runtime: &PluginRuntime, nickname: &str) {
    let Some((server, state)) = runtime.current_server() else {
        return;
    };
    state.lock().session.apply_local_release_all_tangents();
    let broadcast = encode(&PluginCommand::ReleaseAllTangents {
        nickname: nickname.to_owned(),
    });
    if let Some(ts) = runtime.act() {
        runtime
            .push_to_talk
            .release(ts, server, broadcast, std::time::Duration::ZERO);
    }
}
