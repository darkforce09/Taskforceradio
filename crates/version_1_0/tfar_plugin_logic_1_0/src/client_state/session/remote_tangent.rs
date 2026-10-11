//! The apply function of a remote radio press or release (`TANGENT`, `TANGENT_LR`,
//! `TANGENT_DD` plugin commands).
//!
//! **Role:** update the sender's transmit state from a tangent broadcast, count the local
//! player's received transmissions, and say which remote radio sounds to play where, as
//! `processTangentPress` (`reference/tfar_1_0/ts/src/plugin.cpp:879-1020`).
//!
//! **Position:** a module of the crate's `session`; called by the plugin app's plugin-command
//! handler with the session lock held; the reception routing comes in as a closure, which
//! `crate::radio_routing::apply_remote_tangent` supplies.
//!
//! **Signals and state:** changes the sender's tangent, frequency, range, subtype, radio
//! abilities, activity time and plugin flag, and the local player's received transmissions.
//!
//! **Invariants:** for a release the routing is computed before the sender's state is reset,
//! with the sender's abilities as the ignore flags; for a press after it is set. Only a real
//! press (the sender held no tangent) or a real release (it held one) plays a sound; a repeated
//! press changes no sound and no count; a sender counts once while the local player receives it.

use std::time::Instant;

use crate::plugin_commands::{TangentBroadcast, TangentKind, TeamspeakClientId};
use tfar_foundation::arma_values::StereoMode;
use tfar_foundation::geometry::Position;

use crate::client_state::client_data::ClientData;
use crate::client_state::radio_volume::{radio_volume_gain, remote_radio_sound};
use crate::client_state::reception::{HeardVia, IgnoreTangents, ReceivedOn, TangentRadio};
use crate::client_state::session::session_state::SessionState;

#[cfg(test)]
#[path = "tests/remote_tangent.rs"]
mod tests;

/// The isolation sum above which a vehicle stops all sound (`std::clamp(…, 0, 0.99)`).
const MOST_VEHICLE_LOSS: f64 = 0.99;

/// How a tangent broadcast relates to the sender's previous tangent state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TangentTransition {
    /// A press while the sender held no tangent.
    Pressed,
    /// A release while the sender held a tangent.
    Released,
    /// A press while the sender already held a tangent.
    RepeatedPress,
    /// A release while the sender held no tangent.
    RepeatedRelease,
}

impl TangentTransition {
    /// The transition of a broadcast that `is_pressed`, from a sender that `was_transmitting`.
    const fn of(was_transmitting: bool, is_pressed: bool) -> Self {
        match (was_transmitting, is_pressed) {
            (false, true) => Self::Pressed,
            (true, false) => Self::Released,
            (true, true) => Self::RepeatedPress,
            (false, false) => Self::RepeatedRelease,
        }
    }
}

/// Where and how loud one remote radio sound plays (the arguments of `playWavFile`,
/// `plugin.cpp:1000-1002`).
#[derive(Clone, Debug, PartialEq)]
pub struct RemoteSoundPlacement {
    /// The gain: the radio volume curve times the global volume.
    pub gain: f64,
    /// Where the sound comes from; `None` plays it at the listener.
    pub position: Option<Position>,
    /// The sound plays from a loudspeaker.
    pub is_on_ground: bool,
    /// The receiving radio's volume.
    pub volume: f64,
    /// The speaker is under water (its wave height below -1.1 m).
    pub is_underwater: bool,
    /// The vehicle isolation between the listener and a speaker, 0 to 0.99.
    pub vehicle_volume_loss: f64,
    /// The listener is in the speaker's vehicle (always true for a carried radio).
    pub is_same_vehicle: bool,
    /// The ear the receiving radio plays into.
    pub stereo_mode: StereoMode,
}

/// The remote radio sound to play, once per radio that hears the sender.
#[derive(Clone, Debug, PartialEq)]
pub struct RemoteRadioSounds {
    /// The sound, relative to the plugin folder and without `.wav`.
    pub sound: &'static str,
    /// One placement per radio that hears the sender.
    pub placements: Vec<RemoteSoundPlacement>,
}

/// What [`SessionState::apply_remote_tangent`] asks the app to do.
#[derive(Clone, Debug, PartialEq)]
pub struct TangentEffects {
    /// The sender.
    pub client_id: TeamspeakClientId,
    /// How the broadcast relates to the sender's previous state.
    pub transition: TangentTransition,
    /// The start sound of a real press, when a radio of the living player hears it.
    pub play_start: Option<RemoteRadioSounds>,
    /// The end sound of a real release, when a radio of the living player heard it.
    pub play_end: Option<RemoteRadioSounds>,
    /// The local player's received transmissions changed (the `POS` reply's second character).
    pub is_receiving_changed: bool,
    /// Every way the local player hears the sender; the app recomputes the sender's mute with
    /// "heard over radio" set to whether this is empty.
    pub heard: Vec<HeardVia>,
}

/// The wave height below which a speaker plays under water (`UNDERWATER_LEVEL`).
const UNDERWATER_LEVEL: f64 = -1.1;

impl SessionState {
    /// Takes a tangent broadcast from another player at `now`.
    ///
    /// `route` lists how the local player hears a sender: it gets the session, the sender and
    /// the ignore flags, and is called once (before the reset for a release, after the update
    /// for a press). `None` when the sender is unknown, the local player is unknown, or the
    /// broadcast is the local player's own.
    pub fn apply_remote_tangent<R>(
        &mut self,
        tangent: &TangentBroadcast,
        now: Instant,
        route: R,
    ) -> Option<TangentEffects>
    where
        R: FnOnce(&Self, &ClientData, IgnoreTangents) -> Vec<HeardVia>,
    {
        let own_client_id = self.clients.own_client_id()?;
        let sender = self.clients.get(&tangent.nickname)?;
        let client_id = sender.client_id();
        if client_id == own_client_id {
            return None;
        }
        let transition = TangentTransition::of(sender.is_transmitting(), tangent.is_pressed);
        let mut route = Some(route);
        let mut heard = Vec::new();
        if !tangent.is_pressed
            && let Some(route) = route.take()
        {
            heard = route(self, sender, ignore_flags(sender));
        }
        let sender = self.clients.get_mut(&tangent.nickname)?;
        update_sender(sender, tangent, now);
        if tangent.is_pressed
            && let Some(route) = route.take()
        {
            let sender = self.clients.get(&tangent.nickname)?;
            heard = route(self, sender, ignore_flags(sender));
        }
        let placements = self.sound_placements(&heard);
        let is_receiving_changed = self.count_reception(
            transition,
            client_id,
            &tangent.frequency,
            !placements.is_empty(),
        );
        let sounds = |is_start| {
            remote_radio_sound(&tangent.subtype, is_start)
                .filter(|_| !placements.is_empty())
                .map(|sound| RemoteRadioSounds {
                    sound,
                    placements: placements.clone(),
                })
        };
        let (play_start, play_end) = match transition {
            TangentTransition::Pressed => (sounds(true), None),
            TangentTransition::Released => (None, sounds(false)),
            TangentTransition::RepeatedPress | TangentTransition::RepeatedRelease => (None, None),
        };
        Some(TangentEffects {
            client_id,
            transition,
            play_start,
            play_end,
            is_receiving_changed,
            heard,
        })
    }

    /// The placements of the remote radio sound for `heard`: one per carried or speaker radio,
    /// none while the local player is dead.
    fn sound_placements(&self, heard: &[HeardVia]) -> Vec<RemoteSoundPlacement> {
        if !self.game.is_alive {
            return Vec::new();
        }
        let Some(own) = self.clients.own() else {
            return Vec::new();
        };
        heard
            .iter()
            .filter(|via| via.on != ReceivedOn::Intercom)
            .map(|via| {
                let is_on_ground = via.on == ReceivedOn::Ground;
                let (vehicle_volume_loss, is_same_vehicle) = if is_on_ground {
                    (
                        (own.vehicle.isolation + via.vehicle.isolation)
                            .clamp(0.0, MOST_VEHICLE_LOSS),
                        own.vehicle.name == via.vehicle.name,
                    )
                } else {
                    (0.0, true)
                };
                RemoteSoundPlacement {
                    gain: radio_volume_gain(via.volume) * self.game.global_volume,
                    position: via.position,
                    is_on_ground,
                    volume: via.volume,
                    is_underwater: via.wave_z < UNDERWATER_LEVEL,
                    vehicle_volume_loss,
                    is_same_vehicle,
                    stereo_mode: via.stereo_mode,
                }
            })
            .collect()
    }

    /// Counts the sender in or out of the local player's received transmissions; whether the
    /// set changed. A real press heard on a radio counts it in; any release counts it out; a
    /// repeated press changes nothing.
    fn count_reception(
        &mut self,
        transition: TangentTransition,
        sender: TeamspeakClientId,
        frequency: &str,
        is_heard: bool,
    ) -> bool {
        let Some(own) = self.clients.own_mut() else {
            return false;
        };
        match transition {
            TangentTransition::Pressed if is_heard => own.receiving.start(sender, frequency),
            TangentTransition::Released | TangentTransition::RepeatedRelease => {
                own.receiving.stop(sender)
            }
            TangentTransition::Pressed | TangentTransition::RepeatedPress => false,
        }
    }
}

/// The ignore flags of a tangent's routing: the sender's radio abilities.
const fn ignore_flags(sender: &ClientData) -> IgnoreTangents {
    IgnoreTangents {
        short_range: sender.can_use_sw_radio,
        long_range: sender.can_use_lr_radio,
    }
}

/// Writes the broadcast into the sender's state: it runs TFAR, it was active now, its subtype
/// and abilities, and its tangent, frequency and range (0 on a release).
fn update_sender(sender: &mut ClientData, tangent: &TangentBroadcast, now: Instant) {
    sender.mark_plugin_enabled(now);
    sender.last_activity_at = Some(now);
    sender.transmitting_subtype = Some(tangent.subtype.clone());
    if tangent.kind == TangentKind::LongRange {
        sender.can_use_lr_radio = true;
    }
    sender.can_use_sw_radio = tangent.can_use_sw_radio;
    sender.can_use_dd_radio = tangent.can_use_dd_radio;
    if tangent.is_pressed {
        sender.transmitting_over = Some(match tangent.kind {
            TangentKind::LongRange => TangentRadio::LongRange,
            TangentKind::ShortRange | TangentKind::DiverDigital => TangentRadio::ShortRange,
        });
        sender.transmitting_frequency = Some(tangent.frequency.clone());
        sender.range = tangent.range;
    } else {
        sender.transmitting_over = None;
        sender.transmitting_frequency = None;
        sender.range = 0;
    }
}
