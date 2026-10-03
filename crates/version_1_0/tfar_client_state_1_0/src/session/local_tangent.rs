//! The apply functions of the local player's own radio tangent from the game (`TANGENT`,
//! `TANGENT_LR`, `TANGENT_DD`, `RELEASE_ALL_TANGENTS`).
//!
//! **Role:** the state half of the `TANGENT` and `RELEASE_ALL_TANGENTS` cases of
//! `reference/tfar_1_0/ts/src/CommandProcessor.cpp:293-376`: the tangent flag, the own
//! transmit state, the half-duplex transmitting radio, and the local sound and broadcast the
//! app plays and sends once the lock is released.
//!
//! **Position:** a module of the crate's `session`; called by the plugin app's async game
//! message path with the session lock held. The push-to-talk and VAD handling stay in the app,
//! which acts on the returned decisions.
//!
//! **Signals and state:** changes the game data's tangent flag and transmitting radio and the
//! local player's subtype, range, tangent and frequency.
//!
//! **Invariants:** only a change of the tangent flag acts, as the original; `TANGENT_DD`
//! transmits as a short-range radio; a release-all clears the tangent flag and the own transmit
//! state, so the next press is a change again, where the original leaves the flag set and
//! ignores that press.

use tfar_arma_values::{RadioSubtype, StereoMode};
use tfar_game_commands_1_0::prelude::{TangentKind as GameTangentKind, TangentPress};
use tfar_plugin_commands_1_0::{TangentBroadcast, TangentKind};
use tfar_plugin_settings_1_0::PluginSettings;

use crate::radio_volume::{local_radio_sound, radio_volume_gain};
use crate::reception::TangentRadio;
use crate::session::session_state::SessionState;

#[cfg(test)]
#[path = "tests/local_tangent.rs"]
mod tests;

/// The prefix the own frequency carries after a release (`"prev_" + frequency`,
/// `CommandProcessor.cpp:322`).
pub const RELEASED_FREQUENCY_PREFIX: &str = "prev_";

/// The local start or end sound of a tangent change.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalTangentSound {
    /// The sound, relative to the plugin folder and without `.wav`.
    pub sound: &'static str,
    /// The radio volume curve times the global volume; 1 when the frequency is not on a radio
    /// of the list the subtype picks.
    pub gain: f64,
    /// The ear the radio plays into; both when the frequency is not found.
    pub stereo_mode: StereoMode,
}

/// What [`SessionState::apply_local_tangent`] asks the app to do after releasing the lock.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalTangentApplied {
    /// A press (force the microphone on and broadcast now) or a release (broadcast after the
    /// push-to-talk and tangent release delays, then restore push-to-talk or VAD).
    pub is_pressed: bool,
    /// The local sound to play, for a subtype that has one.
    pub sound: Option<LocalTangentSound>,
    /// The tangent broadcast to send to the channel.
    pub broadcast: TangentBroadcast,
}

/// What [`SessionState::apply_local_release_all_tangents`] changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LocalReleaseAllApplied {
    /// The local player held a tangent before the release.
    pub was_tangent_pressed: bool,
}

impl SessionState {
    /// Takes the local player's tangent press or release from the game under `settings`.
    ///
    /// `None` when there is no local player or the tangent flag does not change, as the
    /// original returns early. Otherwise the flag, the local player's subtype and transmit
    /// state (range, radio, frequency; `prev_` + frequency after a release) and, in half-duplex
    /// mode, the transmitting radio are written: a press with a radio class name sets it, a
    /// release clears it.
    pub fn apply_local_tangent(
        &mut self,
        press: &TangentPress,
        settings: &PluginSettings,
    ) -> Option<LocalTangentApplied> {
        let game = &mut self.game;
        let own = self.clients.own_mut()?;
        if game.is_tangent_pressed == press.is_pressed {
            return None;
        }
        game.is_tangent_pressed = press.is_pressed;
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
        let frequencies = if press.subtype == RadioSubtype::Digital {
            &game.sw_frequencies
        } else {
            &game.lr_frequencies
        };
        let sound = local_radio_sound(&press.subtype, press.is_pressed).map(|sound| {
            frequencies.get(&press.frequency).map_or(
                LocalTangentSound {
                    sound,
                    gain: 1.0,
                    stereo_mode: StereoMode::Both,
                },
                |entry| LocalTangentSound {
                    sound,
                    gain: radio_volume_gain(entry.volume) * game.global_volume,
                    stereo_mode: entry.stereo_mode,
                },
            )
        });
        if !settings.full_duplex() {
            if !press.is_pressed {
                game.current_transmitting_radio = None;
            } else if let Some(classname) = &press.radio_classname {
                game.current_transmitting_radio = Some(classname.clone());
            }
        }
        let broadcast = TangentBroadcast {
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
        };
        Some(LocalTangentApplied {
            is_pressed: press.is_pressed,
            sound,
            broadcast,
        })
    }

    /// Takes the game's `RELEASE_ALL_TANGENTS` for the local player: the tangent flag clears,
    /// the local player holds no radio (range 0, no tangent) and no radio is being transmitted
    /// on, so the next press is a change and the half-duplex radio hears again. The app then
    /// releases push-to-talk at once and broadcasts the release.
    pub fn apply_local_release_all_tangents(&mut self) -> LocalReleaseAllApplied {
        let was_tangent_pressed = self.game.is_tangent_pressed;
        self.game.is_tangent_pressed = false;
        self.game.current_transmitting_radio = None;
        if let Some(own) = self.clients.own_mut() {
            own.range = 0;
            own.transmitting_over = None;
        }
        LocalReleaseAllApplied {
            was_tangent_pressed,
        }
    }
}

/// The plugin-command tangent kind of a game tangent kind.
const fn broadcast_kind(kind: GameTangentKind) -> TangentKind {
    match kind {
        GameTangentKind::ShortRange => TangentKind::ShortRange,
        GameTangentKind::LongRange => TangentKind::LongRange,
        GameTangentKind::DiverDigital => TangentKind::DiverDigital,
    }
}
