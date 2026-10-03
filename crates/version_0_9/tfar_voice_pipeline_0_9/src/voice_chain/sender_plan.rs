//! The plan of one sender's frame: every gain, branch and direction, read before any sound
//! state is borrowed.
//!
//! **Role:** read the server state once per frame and decide what the stereo function
//! (`reference/tfar_0_9_12/ts/src/plugin.cpp:2687-2797`) does with the frame: the direct
//! voice's branch and level, and per reception its effect gain and error, its loudspeaker
//! stages and where each sound comes from.
//!
//! **Position:** called by `VoiceProcessor` under the server lock, before the render steps in
//! `direct_voice.rs` and `radio_reception.rs` borrow the sender's sound state; uses the
//! reception rules of `tfar_radio_routing_0_9`.
//!
//! **Signals and state:** none of its own; reads the server state and fills the scratch
//! reception and route lists on the TeamSpeak audio thread; their radio ids are shared with the
//! server state, so planning copies no text.
//!
//! **Invariants:** the direct voice is planned only for another player within their voice
//! volume; a muffled voice is planned only when both sides can speak; the radio effect follows
//! the transmitter's subtype; the underwater effect's error uses the raw distance and every
//! other effect's the effective distance.

use std::sync::Arc;

use tfar_client_state_0_9::prelude::{Position, StereoMode};
use tfar_client_state_0_9::{ClientData, RadioEffectKind, ServerRadioData, VehicleMuffling};
use tfar_hrtf::ListenerRelativeDirection;
use tfar_radio_routing_0_9::{
    HeardOn, HeardVia, IgnoreTangents, effect_error_from_distance, effective_distance,
    is_over_radio_into, raw_distance,
};
use tfar_spatial_audio::{ILD_COEFFICIENT_0_9, stereo_pan_gains};

use crate::distance_gain::{radio_volume_multiplier, volume_from_distance};
use crate::voice_gains::{
    CANT_SPEAK_GAIN, DIVER_RADIO_EFFECT_GAIN, PHONE_GAIN, RADIO_EFFECT_GAIN, vehicle_loss_gain,
};

#[cfg(test)]
#[path = "tests/sender_plan.rs"]
mod tests;

/// Where a sound comes from, for the HRTF and for the ILD fallback.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SoundPlacement {
    /// The direction from the listener's head, for an HRTF source.
    pub(crate) direction: ListenerRelativeDirection,
    /// The 0.9 simple ILD gains `(left, right)`, used when no HRTF source exists.
    pub(crate) ild_gains: (f32, f32),
}

/// The distance level of a sound the listener can hear: a plain gain, or the vehicle low-pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DistanceLevel {
    /// A plain gain.
    Plain {
        /// The distance gain.
        gain: f64,
    },
    /// The vehicle low-pass with this gain.
    VehicleMuffled {
        /// The distance gain times the vehicle level loss.
        gain: f64,
    },
}

/// The level branch of a direct voice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DirectVoiceLevel {
    /// Both sides can speak: the distance level.
    Heard(DistanceLevel),
    /// Either side cannot speak: the 100 Hz can't-speak low-pass with this gain.
    CantSpeak {
        /// The muffled distance gain times [`CANT_SPEAK_GAIN`].
        gain: f64,
    },
}

/// The direct voice of a sender in voice range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DirectVoicePlan {
    /// The branch and its gain.
    pub(crate) level: DirectVoiceLevel,
    /// Where the sender stands.
    pub(crate) placement: SoundPlacement,
}

/// The stages a loudspeaker adds after the radio effect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LoudspeakerPlan {
    /// The distance gain, plain or through the vehicle low-pass.
    pub(crate) level: DistanceLevel,
    /// Whether the loudspeaker sounds under water.
    pub(crate) is_under_water: bool,
    /// Where the loudspeaker stands.
    pub(crate) placement: SoundPlacement,
}

/// One reception of the sender's radio, planned.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlannedRoute {
    /// The radio id that keys the reception's sound state, shared with the server state.
    pub(crate) radio_id: Arc<str>,
    /// The ear the receiving radio plays into.
    pub(crate) stereo_mode: StereoMode,
    /// The gain into the radio effect or the phone band-pass.
    pub(crate) effect_gain: f64,
    /// The raw distance error the effect degrades by.
    pub(crate) effect_error: f64,
    /// The loudspeaker stages, for a reception on a loudspeaker.
    pub(crate) loudspeaker: Option<LoudspeakerPlan>,
}

/// Everything one sender's frame needs besides the reception list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SenderPlan {
    /// The sender's voice volume multiplier, the first gain of the chain.
    pub(crate) voice_volume_multiplier: f64,
    /// The direct voice, when the listener is in its range.
    pub(crate) direct_voice: Option<DirectVoicePlan>,
    /// The effect the sender's subtype selects for every reception.
    pub(crate) effect: RadioEffectKind,
    /// The gain on everything the plugin outputs.
    pub(crate) global_volume: f64,
}

/// Plans the frame of `sender` (the entry of `sender_nickname`) for `listener`, the local
/// player's entry: refills `heard` with its receptions and `routes` with their plans, in the
/// routing's order, both reused without allocating once they hold every reception.
pub(crate) fn plan_sender(
    server: &ServerRadioData,
    sender_nickname: &str,
    sender: &ClientData,
    listener: &ClientData,
    heard: &mut Vec<HeardVia>,
    routes: &mut Vec<PlannedRoute>,
) -> SenderPlan {
    let should_player_hear = sender.can_speak && server.can_speak;
    let distance = raw_distance(server, sender);
    let is_in_voice_range =
        sender.client_id != listener.client_id && distance <= f64::from(sender.voice_volume);
    let direct_voice = is_in_voice_range.then(|| DirectVoicePlan {
        level: direct_voice_level(listener, sender, distance, should_player_hear),
        placement: placement(server, listener, sender.position),
    });
    let effect = RadioEffectKind::for_subtype(&sender.subtype);
    let reception = ReceptionFacts {
        effect,
        radio_distance: effective_distance(server, sender),
        listener_distance: listener.position.distance_to(sender.position),
        should_player_hear,
    };
    is_over_radio_into(
        server,
        sender_nickname,
        sender,
        listener,
        IgnoreTangents::NONE,
        heard,
    );
    routes.clear();
    routes.extend(
        heard
            .iter()
            .map(|heard| plan_route(server, sender, listener, &reception, heard)),
    );
    SenderPlan {
        voice_volume_multiplier: sender.voice_volume_multiplier,
        direct_voice,
        effect,
        global_volume: server.global_volume,
    }
}

/// What every reception of one sender shares.
struct ReceptionFacts {
    effect: RadioEffectKind,
    /// The effective distance (terrain, multiplier) the short- and long-range errors use.
    radio_distance: f64,
    /// The raw distance between the two entries, which the underwater error uses
    /// (`distance(data->clientPosition, myData->clientPosition)`, plugin.cpp:2748).
    listener_distance: f64,
    should_player_hear: bool,
}

/// The direct voice's branch: plain, muffled by a vehicle, or the can't-speak low-pass
/// (plugin.cpp:2698-2725).
fn direct_voice_level(
    listener: &ClientData,
    sender: &ClientData,
    distance: f64,
    should_player_hear: bool,
) -> DirectVoiceLevel {
    if !should_player_hear {
        return DirectVoiceLevel::CantSpeak {
            gain: volume_from_distance(distance, false, sender.voice_volume, 1.0) * CANT_SPEAK_GAIN,
        };
    }
    let muffling = VehicleMuffling::between(&listener.vehicle, &sender.vehicle);
    DirectVoiceLevel::Heard(distance_level(
        distance,
        true,
        sender.voice_volume,
        muffling,
    ))
}

/// The distance level of a sound `distance` metres away that carries `reach` metres, through
/// the vehicle low-pass when `muffling` muffles it.
fn distance_level(
    distance: f64,
    should_player_hear: bool,
    reach: i32,
    muffling: VehicleMuffling,
) -> DistanceLevel {
    match muffling.muffling_isolation() {
        None => DistanceLevel::Plain {
            gain: volume_from_distance(distance, should_player_hear, reach, 1.0),
        },
        Some(isolation) => DistanceLevel::VehicleMuffled {
            gain: volume_from_distance(distance, should_player_hear, reach, 1.0 - isolation)
                * vehicle_loss_gain(isolation),
        },
    }
}

/// The plan of one reception (plugin.cpp:2730-2794).
fn plan_route(
    server: &ServerRadioData,
    sender: &ClientData,
    listener: &ClientData,
    reception: &ReceptionFacts,
    heard: &HeardVia,
) -> PlannedRoute {
    let volume = radio_volume_multiplier(heard.volume);
    let error_over = |distance: f64| {
        effect_error_from_distance(heard.over, distance, sender.range, server.waves_level)
    };
    let (effect_gain, effect_error) = match reception.effect {
        RadioEffectKind::Personal | RadioEffectKind::LongRange => (
            volume * RADIO_EFFECT_GAIN,
            error_over(reception.radio_distance),
        ),
        RadioEffectKind::UnderWater => (
            radio_volume_multiplier(server.dd_volume_level) * DIVER_RADIO_EFFECT_GAIN,
            error_over(reception.listener_distance),
        ),
        RadioEffectKind::Phone => (volume * PHONE_GAIN, 0.0),
        RadioEffectKind::Silent => (0.0, 0.0),
    };
    let loudspeaker = (heard.on == HeardOn::Loudspeaker).then(|| LoudspeakerPlan {
        level: distance_level(
            server.my_position.distance_to(heard.position),
            reception.should_player_hear,
            loudspeaker_reach(heard.volume, server.speaker_distance),
            VehicleMuffling::between(&listener.vehicle, &heard.vehicle),
        ),
        is_under_water: heard.is_under_water(),
        placement: placement(server, listener, heard.position),
    });
    PlannedRoute {
        radio_id: Arc::clone(&heard.radio_id),
        stereo_mode: heard.stereo_mode,
        effect_gain,
        effect_error,
        loudspeaker,
    }
}

/// The metres a loudspeaker at `volume` carries: `volume / 10 · speaker_distance`, truncated to
/// whole metres as the original's `int` parameter takes it (plugin.cpp:2772).
#[expect(
    clippy::cast_possible_truncation,
    reason = "the original passes the float reach to an int parameter, truncating toward zero"
)]
fn loudspeaker_reach(volume: i32, speaker_distance: f64) -> i32 {
    (f64::from(volume) / 10.0 * speaker_distance) as i32
}

/// Where a sound at `source` comes from for `listener`, who looks along their view angle.
fn placement(server: &ServerRadioData, listener: &ClientData, source: Position) -> SoundPlacement {
    SoundPlacement {
        direction: ListenerRelativeDirection::from_arma(
            server.my_position,
            listener.view_angle,
            source,
        ),
        ild_gains: stereo_pan_gains(
            server.my_position.direction_to(source),
            listener.view_angle,
            ILD_COEFFICIENT_0_9,
        ),
    }
}
