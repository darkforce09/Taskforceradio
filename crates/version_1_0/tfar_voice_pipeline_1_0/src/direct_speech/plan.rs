//! The direct-speech plan: whether a sender is heard directly, muffled how, from where.
//!
//! **Role:** decide the direct-speech branch of `processVoiceData`
//! (`reference/tfar_1_0/ts/src/plugin.cpp:547-597`) for one frame: the range gate, the
//! muffling (objects, another vehicle, unable to speak) with its gain, and the placement, whose
//! level is the X3DAudio distance curve and voice cone of `helpers.cpp:30-167` rebuilt on
//! `tfar_spatial_audio`.
//!
//! **Position:** a module of the crate's `direct_speech`; called by the frame plan and the DSP
//! needs with a [`Scene`].
//!
//! **Signals and state:** none; a pure function of the scene.
//!
//! **Invariants:** the local player, a spectating sender and a sender the spectator rules keep
//! away are never heard directly; the cone measures the sender's facing towards the listener.

use tfar_geometry::Direction;
use tfar_spatial_audio::{DistanceCurve, EmitterCone};

use crate::distance_gain::{
    CANT_SPEAK_GAIN, NO_VEHICLE_LOSS, gain_as_f32, vehicle_loss_gain, vehicle_volume_loss,
    volume_attenuation,
};
use crate::placement::{Placement, hrtf_makeup_gain};
use crate::scene::Scene;
use crate::sender_gate::SpectatorView;

/// How far beyond the sender's voice range direct speech is still processed (`voiceVolume +
/// 15`).
pub(crate) const DIRECT_SPEECH_RANGE_MARGIN: f64 = 15.0;

/// The metres each object between the two adds to the distance (`2 * objectInterception`).
const METRES_PER_OBJECT: f64 = 2.0;

/// How direct speech is muffled before it is placed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Muffling {
    /// Clear speech: nothing to muffle; the frame is still kept for the object look-back.
    Clear,
    /// Objects between the two: the object low-pass for `object_count`, run with the previous
    /// frame as look-back.
    ObjectInterception {
        /// The objects between the two.
        object_count: i32,
    },
    /// The two sit in different vehicles: the vehicle low-pass at `loss`, at `gain`.
    Vehicle {
        /// The isolation loss, which also keys the filter.
        loss: f64,
        /// The distance and isolation gain.
        gain: f32,
    },
    /// One of the two cannot speak: the 100 Hz low-pass at `gain`.
    CantSpeak {
        /// The short-reach distance gain times [`CANT_SPEAK_GAIN`].
        gain: f32,
    },
}

/// The direct-speech processing of one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DirectSpeechPlan {
    /// The muffling step.
    pub(crate) muffling: Muffling,
    /// The direction and level.
    pub(crate) placement: Placement,
}

/// The vehicle state of the two: the isolation loss and whether they share a vehicle
/// (`plugin.cpp:538-541`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VehicleRelation {
    /// The isolation between them, 0 to 0.99.
    pub(crate) loss: f64,
    /// Both sit in the same vehicle (names equal and not `no`).
    pub(crate) is_same_vehicle: bool,
}

impl VehicleRelation {
    /// The relation of the scene's two players.
    pub(crate) fn of(scene: &Scene<'_>) -> Self {
        let (mine, his) = (&scene.listener.vehicle, &scene.sender.vehicle);
        Self {
            loss: vehicle_volume_loss(mine.isolation, his.isolation),
            is_same_vehicle: mine.is_in_vehicle() && mine.name == his.name,
        }
    }

    /// Whether the sound reaches the listener unmuffled by vehicles.
    pub(crate) fn is_unmuffled(self) -> bool {
        self.loss < NO_VEHICLE_LOSS || self.is_same_vehicle
    }
}

/// The distance direct speech is judged by: the straight distance plus 2 m per object.
pub(crate) fn distance_with_objects(scene: &Scene<'_>) -> f64 {
    METRES_PER_OBJECT.mul_add(
        f64::from(scene.sender.object_interception),
        scene.distance(),
    )
}

/// The direct-speech plan of the scene, or `None` when the sender is not heard directly: the
/// sender is the local player or spectates, the spectator rules keep them away, or they stand
/// beyond their voice range plus 15 m (objects counting 2 m each).
pub(crate) fn plan_direct_speech(
    scene: &Scene<'_>,
    view: SpectatorView,
) -> Option<DirectSpeechPlan> {
    let distance = distance_with_objects(scene);
    let voice_volume = scene.voice_volume();
    if scene.is_own_voice
        || scene.sender.is_spectating
        || view.is_not_hearable
        || distance > voice_volume + DIRECT_SPEECH_RANGE_MARGIN
    {
        return None;
    }
    Some(DirectSpeechPlan {
        muffling: muffling(scene, distance),
        placement: Placement::new(
            scene.listener_position,
            scene.listener_view,
            scene.sender_position,
            voice_level(scene),
            hrtf_makeup_gain(),
        ),
    })
}

/// The muffling of the scene's speech at `distance` (objects included).
fn muffling(scene: &Scene<'_>, distance: f64) -> Muffling {
    let voice_volume = scene.voice_volume();
    let vehicles = VehicleRelation::of(scene);
    if !scene.should_hear {
        let gain = volume_attenuation(distance, false, voice_volume, 1.0);
        return Muffling::CantSpeak {
            gain: gain_as_f32(gain) * CANT_SPEAK_GAIN,
        };
    }
    if vehicles.is_unmuffled() {
        let object_count = scene.sender.object_interception;
        return if !vehicles.is_same_vehicle && object_count > 0 {
            Muffling::ObjectInterception { object_count }
        } else {
            Muffling::Clear
        };
    }
    let audible = 1.0 - vehicles.loss;
    let gain = volume_attenuation(distance, true, voice_volume, audible)
        * vehicle_loss_gain(vehicles.loss);
    Muffling::Vehicle {
        loss: vehicles.loss,
        gain: gain_as_f32(gain),
    }
}

/// The level of direct speech: the 25-point voice curve at the straight distance, scaled by the
/// sender's voice range, times the voice cone when `voiceCone` is on.
///
/// The original computes this with X3DAudio and swaps the roles on purpose
/// (`helpers.cpp:46-110`): its listener stands at the sender facing the local player's view,
/// and its emitter stands at the local player facing the sender's view with x and y negated.
/// The cone therefore measures the angle between that negated facing and the line from the
/// local player to the sender, so a sender who faces the local player gets the inner volume.
pub(crate) fn voice_level(scene: &Scene<'_>) -> f64 {
    let voice_volume = scene.voice_volume();
    let should_hear = scene.should_hear;
    let curve_gain = DistanceCurve::tfar_voice(voice_volume, |distance| {
        volume_attenuation(distance, should_hear, voice_volume, 1.0)
    })
    .map_or(0.0, |curve| curve.gain_at(scene.distance(), voice_volume));
    if !scene.settings.voice_cone() {
        return curve_gain;
    }
    let [x, y, z] = scene.sender.view_direction.to_array();
    let emitter_front = Direction::from_array([-x, -y, z]).unwrap_or(Direction::ZERO);
    let emitter_to_listener = scene.listener_position.direction_to(scene.sender_position);
    curve_gain * EmitterCone::TFAR_VOICE.gain(emitter_front, emitter_to_listener)
}
