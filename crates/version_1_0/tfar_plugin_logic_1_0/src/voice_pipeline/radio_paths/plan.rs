//! The radio-path plan: how each reception of a sender is processed this frame.
//!
//! **Role:** turn every `HeardVia` of `is_over_radio` into a [`RoutePlan`], as the loop of
//! `processVoiceData` decides it (`reference/tfar_1_0/ts/src/plugin.cpp:600-737`): the effect
//! by the sender's subtype and its error level for a carried radio, the speaker band-pass,
//! distance gain, vehicle and underwater muffling and placement for a speaker radio, and the
//! intercom volume with its ducking.
//!
//! **Position:** a module of the crate's `radio_paths`; called by the frame plan and the DSP
//! needs with a [`Scene`].
//!
//! **Signals and state:** none; a pure function of the scene and the receptions.
//!
//! **Invariants:** one plan per reception, in the receptions' order; an unknown subtype is
//! silence; the intercom is never heard from the local player's own microphone.

use crate::client_state::prelude::{
    HeardVia, RadioSubtype, ReceivedOn, SendingRadio, radio_volume_gain,
};
use crate::radio_routing::{
    capped_by_antenna, effect_error_from_distance, underwater_effect_error,
};

use crate::voice_pipeline::distance_gain::{
    NO_VEHICLE_LOSS, UNDERWATER_LEVEL, gain_as_f32, speaker_audible_distance, vehicle_loss_gain,
    vehicle_volume_loss, volume_attenuation,
};
use crate::voice_pipeline::placement::Placement;
use crate::voice_pipeline::scene::Scene;

/// The share of the radio volume a radio effect plays at (`volumeLevel * 0.35f`).
const RADIO_EFFECT_GAIN: f64 = 0.35;

/// The gain of the phone band-pass over the radio volume (`volumeLevel * 10.0f`).
const PHONE_GAIN: f64 = 10.0;

/// The factor a lowered headset leaves of a carried radio or the intercom.
const HEADSET_LOWERED_GAIN: f64 = 0.1;

/// The radio volume above which a long-range reception ducks the intercom (`it.volume > 2`).
const DUCKING_VOLUME: f64 = 2.0;

/// The sound a carried radio makes, by the sender's subtype (`plugin.cpp:619-675`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CarriedRadioSound {
    /// A short-range radio heard above water: the personal radio effect.
    Personal,
    /// A short-range radio of a sender heard as a diver radio: the underwater effect.
    UnderWater,
    /// An airborne radio: the airborne effect.
    Airborne,
    /// A long-range radio: the long-range effect.
    LongRange,
    /// A phone: the phone band-pass.
    Phone,
    /// Any other subtype (`dd`, unknown, none): silence.
    Silence,
}

/// How a speaker radio's sound is muffled before its effect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SpeakerMuffling {
    /// No vehicle between the speaker and the listener: the distance gain only.
    Clear {
        /// The distance gain.
        gain: f32,
    },
    /// A vehicle between them: the vehicle low-pass at `loss`, at `gain`.
    Vehicle {
        /// The isolation loss, which also keys the filter.
        loss: f64,
        /// The distance and isolation gain.
        gain: f32,
    },
}

/// The processing of a reception on a speaker radio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SpeakerPlan {
    /// The distance and vehicle step after the band-pass.
    pub(crate) muffling: SpeakerMuffling,
    /// The speaker sits under water: the 100 Hz low-pass follows.
    pub(crate) is_under_water: bool,
    /// The distance error the personal radio effect is set to.
    pub(crate) distance_error: f64,
    /// The radio effect's gain.
    pub(crate) effect_gain: f32,
    /// Where the speaker is heard from.
    pub(crate) placement: Placement,
}

/// The processing of one reception.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum RoutePlan {
    /// Not heard this frame: the intercom from the microphone, or a speaker without position.
    Skipped,
    /// A radio the listener carries.
    Carried {
        /// The sound by the sender's subtype.
        sound: CarriedRadioSound,
        /// The distance error the effect is set to.
        distance_error: f64,
        /// The effect's (or the phone band-pass's) gain.
        gain: f32,
    },
    /// A speaker radio.
    Speaker(SpeakerPlan),
    /// The vehicle intercom at `gain`.
    Intercom {
        /// The intercom volume after headset and ducking.
        gain: f32,
    },
}

/// Fills `plans` with one plan per reception in `routes`, in order. `is_from_microphone` marks
/// the captured-voice path, where the intercom is skipped so the local player does not hear
/// their own direct speech over it.
pub(crate) fn plan_routes(
    scene: &Scene<'_>,
    routes: &[HeardVia],
    is_from_microphone: bool,
    plans: &mut Vec<RoutePlan>,
) {
    plans.clear();
    let radio_distance = scene
        .listener
        .effective_distance_to(scene.sender, scene.game, scene.now);
    let is_intercom_ducked = is_long_range_reception_active(scene, routes);
    for route in routes {
        let plan = match route.on {
            ReceivedOn::ShortRange | ReceivedOn::LongRange => {
                plan_carried_radio(scene, route, radio_distance)
            }
            ReceivedOn::Ground => plan_speaker(scene, route, radio_distance),
            ReceivedOn::Intercom if is_from_microphone => RoutePlan::Skipped,
            ReceivedOn::Intercom => plan_intercom(scene, is_intercom_ducked),
        };
        plans.push(plan);
    }
}

/// A carried radio: headset lowering, then the effect by the sender's subtype.
fn plan_carried_radio(scene: &Scene<'_>, route: &HeardVia, radio_distance: f64) -> RoutePlan {
    let mut volume_level = radio_volume_gain(route.volume);
    if scene.settings.headset_lowered() {
        volume_level *= HEADSET_LOWERED_GAIN;
    }
    let range = scene.sender.range;
    let capped_error = capped_by_antenna(
        effect_error_from_distance(route.over, radio_distance, range),
        route.antenna_loss,
    );
    let effect_gain = gain_as_f32(volume_level * RADIO_EFFECT_GAIN);
    let (sound, distance_error, gain) = match scene.sender.transmitting_subtype.as_ref() {
        Some(RadioSubtype::Digital) if route.over == SendingRadio::ShortRange => {
            (CarriedRadioSound::Personal, capped_error, effect_gain)
        }
        Some(RadioSubtype::Digital) => {
            let underwater_error = underwater_effect_error(
                scene.listener_position,
                scene.sender_position,
                range,
                scene.game.waves_level,
                route.antenna_loss,
            );
            (CarriedRadioSound::UnderWater, underwater_error, effect_gain)
        }
        Some(RadioSubtype::Airborne) => (CarriedRadioSound::Airborne, capped_error, effect_gain),
        Some(RadioSubtype::DigitalLongRange) => {
            (CarriedRadioSound::LongRange, capped_error, effect_gain)
        }
        Some(RadioSubtype::Phone) => (
            CarriedRadioSound::Phone,
            0.0,
            gain_as_f32(volume_level * PHONE_GAIN),
        ),
        Some(RadioSubtype::DiverDigital | RadioSubtype::Unknown(_)) | None => {
            (CarriedRadioSound::Silence, 0.0, 0.0)
        }
    };
    RoutePlan::Carried {
        sound,
        distance_error,
        gain,
    }
}

/// The isolation between the listener and a speaker radio: none when the speaker is in the
/// listener's vehicle (both outside counts as the same, `no`), else the clamped sum
/// (`plugin.cpp:678-684`).
pub(crate) fn speaker_vehicle_loss(scene: &Scene<'_>, route: &HeardVia) -> f64 {
    let mine = &scene.listener.vehicle;
    if mine.name == route.vehicle.name {
        0.0
    } else {
        vehicle_volume_loss(mine.isolation, route.vehicle.isolation)
    }
}

/// A speaker radio (`plugin.cpp:677-714`). The vehicle muffling depends on the speaker's
/// vehicle alone; the original also skipped it whenever the sender shared the listener's
/// vehicle, though the sound comes from the speaker.
fn plan_speaker(scene: &Scene<'_>, route: &HeardVia, radio_distance: f64) -> RoutePlan {
    let Some(position) = route.position else {
        return RoutePlan::Skipped;
    };
    let distance = scene.listener_position.distance_to(position);
    let reach = speaker_audible_distance(route.volume, scene.game.speaker_distance);
    let loss = speaker_vehicle_loss(scene, route);
    let muffling = if loss < NO_VEHICLE_LOSS {
        SpeakerMuffling::Clear {
            gain: gain_as_f32(volume_attenuation(distance, scene.should_hear, reach, 1.0)),
        }
    } else {
        let gain = volume_attenuation(distance, scene.should_hear, reach, 1.0 - loss)
            * vehicle_loss_gain(loss);
        SpeakerMuffling::Vehicle {
            loss,
            gain: gain_as_f32(gain),
        }
    };
    RoutePlan::Speaker(SpeakerPlan {
        muffling,
        is_under_water: route.wave_z < UNDERWATER_LEVEL,
        distance_error: capped_by_antenna(
            effect_error_from_distance(route.over, radio_distance, scene.sender.range),
            route.antenna_loss,
        ),
        effect_gain: gain_as_f32(radio_volume_gain(route.volume) * RADIO_EFFECT_GAIN),
        placement: Placement::new(
            scene.listener_position,
            scene.listener_view,
            position,
            1.0,
            1.0,
        ),
    })
}

/// The intercom: its volume setting, a tenth with the headset lowered, ducked by
/// `intercomDucking` while a long-range transmission is received (`plugin.cpp:715-732`).
fn plan_intercom(scene: &Scene<'_>, is_ducked: bool) -> RoutePlan {
    let settings = scene.settings;
    let mut gain = settings.intercom_volume();
    if settings.headset_lowered() {
        gain *= gain_as_f32(HEADSET_LOWERED_GAIN);
    }
    if is_ducked {
        gain *= 1.0 - settings.intercom_ducking();
    }
    RoutePlan::Intercom { gain }
}

/// Whether the listener receives a long-range transmission above volume 2 now, which ducks the
/// intercom.
///
/// The original looks only among the receptions of the intercom's own sender
/// (`plugin.cpp:723-727`), but the intercom is listed only for a sender holding no tangent and
/// a long-range reception only for one holding a tangent, so it never ducked. Here any sender
/// the listener receives counts: one of this sender's receptions on a long-range radio, or a
/// received frequency on one of the listener's long-range radios set above volume 2.
fn is_long_range_reception_active(scene: &Scene<'_>, routes: &[HeardVia]) -> bool {
    let is_listed = routes
        .iter()
        .any(|route| route.on == ReceivedOn::LongRange && route.volume > DUCKING_VOLUME);
    let listener = scene.listener;
    let is_received = listener.can_use_lr_radio
        && listener.receiving_frequencies().any(|frequency| {
            scene
                .game
                .lr_frequencies
                .get(frequency)
                .is_some_and(|entry| entry.volume > DUCKING_VOLUME)
        });
    is_listed || is_received
}
