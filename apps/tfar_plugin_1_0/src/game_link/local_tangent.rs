//! The local player's radio tangent from the game (`TANGENT`, `TANGENT_LR`, `TANGENT_DD`,
//! `RELEASE_ALL_TANGENTS`): the own transmit state, the local start and end sounds, the
//! half-duplex radio, push-to-talk and the broadcast to the channel.
//!
//! **Role:** the `TANGENT` and `RELEASE_ALL_TANGENTS` cases of `reference/tfar_1_0/ts/src/
//! CommandProcessor.cpp:293-387`.
//!
//! **Position:** called by the async message path on the link's async worker.
//!
//! **Signals and state:** the current server's game data and own client under its lock; the
//! push-to-talk state of the runtime.
//!
//! **Invariants:** only a change of the tangent state acts; `RELEASE_ALL_TANGENTS` also clears the
//! own tangent, so the next press is a change again.

use tfar_arma_values::StereoMode;
use tfar_client_state_1_0::prelude::{TangentBroadcast, TangentRadio, radio_volume_gain};
use tfar_game_commands_1_0::prelude::{TangentKind as GameTangentKind, TangentPress};
use tfar_plugin_commands_1_0::prelude::{PluginCommand, TangentKind, encode};

use crate::audio::radio_sounds::local_radio_sound;
use crate::runtime::PluginRuntime;

/// The prefix the own frequency carries after a release (`"prev_" + frequency`).
const RELEASED_FREQUENCY_PREFIX: &str = "prev_";

/// The plugin-command tangent kind of a game tangent kind.
const fn broadcast_kind(kind: GameTangentKind) -> TangentKind {
    match kind {
        GameTangentKind::ShortRange => TangentKind::ShortRange,
        GameTangentKind::LongRange => TangentKind::LongRange,
        GameTangentKind::DiverDigital => TangentKind::DiverDigital,
    }
}

/// What a tangent change asks for once the lock is released.
struct TangentWork {
    broadcast: String,
    sound: Option<(String, f64, StereoMode)>,
}

/// A local tangent press or release.
pub fn apply_local_tangent(runtime: &PluginRuntime, press: &TangentPress) {
    let Some((server, state)) = runtime.current_server() else {
        return;
    };
    let settings = runtime.settings();
    let work = {
        let mut state = state.lock();
        let game = &mut state.session.game;
        if game.is_tangent_pressed == press.is_pressed {
            return;
        }
        game.is_tangent_pressed = press.is_pressed;
        let global_volume = game.global_volume;
        let frequencies = if press.subtype == tfar_arma_values::RadioSubtype::Digital {
            &game.sw_frequencies
        } else {
            &game.lr_frequencies
        };
        let sound = local_radio_sound(&press.subtype, press.is_pressed).map(|sound| {
            match frequencies.get(&press.frequency) {
                Some(entry) => (
                    sound,
                    radio_volume_gain(entry.volume) * global_volume,
                    entry.stereo_mode,
                ),
                None => (sound, 1.0, StereoMode::Both),
            }
        });
        if !settings.full_duplex() {
            game.current_transmitting_radio = if press.is_pressed {
                press.radio_classname.clone()
            } else {
                None
            };
        }
        let Some(own) = state.session.clients.own_mut() else {
            return;
        };
        own.transmitting_subtype = Some(press.subtype.clone());
        if press.is_pressed {
            own.range = press.range;
            own.transmitting_over = Some(if press.kind.is_long_range() {
                TangentRadio::LongRange
            } else {
                TangentRadio::ShortRange
            });
            own.transmitting_frequency = Some(press.frequency.clone());
        } else {
            own.range = 0;
            own.transmitting_over = None;
            own.transmitting_frequency =
                Some(format!("{RELEASED_FREQUENCY_PREFIX}{}", press.frequency));
        }
        let broadcast = encode(&PluginCommand::Tangent(TangentBroadcast {
            kind: broadcast_kind(press.kind),
            is_pressed: press.is_pressed,
            frequency: press.frequency.clone(),
            range: press.range,
            range_text: press.range_text.clone(),
            subtype: press.subtype.clone(),
            radio_classname: press.radio_classname.clone(),
            can_use_sw_radio: own.can_use_sw_radio,
            can_use_dd_radio: own.can_use_dd_radio,
            nickname: own.nickname().to_owned(),
        }));
        TangentWork { broadcast, sound }
    };
    if let Some((sound, gain, stereo_mode)) = &work.sound {
        runtime.sounds.play_local(sound, *gain, *stereo_mode);
    }
    let Some(ts) = runtime.act() else {
        return;
    };
    if press.is_pressed {
        runtime.push_to_talk.press(ts, server, &work.broadcast);
    } else {
        let delay = runtime
            .push_to_talk_delay()
            .saturating_add(settings.tangent_release_delay());
        runtime
            .push_to_talk
            .release(ts, server, work.broadcast, delay);
    }
}

/// `RELEASE_ALL_TANGENTS`: every own radio is released at once and the channel told.
pub fn release_all_tangents(runtime: &PluginRuntime, nickname: &str) {
    let Some((server, state)) = runtime.current_server() else {
        return;
    };
    {
        let mut state = state.lock();
        state.session.game.is_tangent_pressed = false;
        if let Some(own) = state.session.clients.own_mut() {
            own.range = 0;
            own.transmitting_over = None;
        }
    }
    let broadcast = encode(&PluginCommand::ReleaseAllTangents {
        nickname: nickname.to_owned(),
    });
    if let Some(ts) = runtime.act() {
        runtime
            .push_to_talk
            .release(ts, server, broadcast, std::time::Duration::ZERO);
    }
}
