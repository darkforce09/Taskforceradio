//! `tfar_spatial_audio` against F3DAudio: the 1.0 voice distance curve for distances from 0 to
//! twice the scaler, the voice cone at 72 angles, and the two together in the scene of the 1.0
//! `applyILD` (`reference/tfar_1_0/ts/src/helpers.cpp`).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "the scene crosses into F3DAudio as f32, the precision X3DAudio computes in"
)]
#![allow(
    clippy::unwrap_used,
    reason = "helpers outside the test functions stop the test on a failed call, as the tests do"
)]

use core::f64::consts::{FRAC_PI_2, TAU};

use tfar_geometry::Direction;
use tfar_golden_vectors::{OracleCone, OracleSpatialScene, f3daudio_calculate};
use tfar_spatial_audio::{DistanceCurve, EmitterCone};

/// How far our gain may lie from F3DAudio's volume.
const GAIN_TOLERANCE: f64 = 1.0e-5;

/// The X3DAudio parameter checks the 1.0 voice curve fails: five points past distance 1 and a last
/// point that is not at distance 1.
const VOICE_CURVE_FAILED_CHECKS: u32 = 6;

/// The 1.0 `volumeAttenuation` (`reference/tfar_1_0/ts/src/helpers.hpp`) for a player who can
/// hear, in `f32` as the original computes it.
fn volume_attenuation(distance: f32, max_audible: f32) -> f32 {
    if distance <= 1.0 {
        return 1.0;
    }
    let gain = 10.0_f32.powf(((distance / (max_audible * 2.0)) * -60.0) / 20.0);
    if gain < 0.001 { 0.0 } else { gain.min(1.0) }
}

/// The voice curve of an emitter of `voice_volume`, as `tfar_spatial_audio` builds it, with the
/// gains rounded to the `f32` the original curve stores.
fn our_voice_curve(voice_volume: f64) -> DistanceCurve {
    DistanceCurve::tfar_voice(voice_volume, |metres| {
        f64::from(volume_attenuation(metres as f32, voice_volume as f32))
    })
    .unwrap()
}

/// The same curve as `(distance, gain)` points in `f32`, the way `applyILD` writes them.
fn original_voice_curve(curve: &DistanceCurve) -> Vec<(f32, f32)> {
    curve
        .points()
        .iter()
        .map(|point| (point.distance as f32, point.gain as f32))
        .collect()
}

/// A listener at `listener_position` facing +y, and an emitter at the origin facing `front`.
fn scene(
    listener_position: [f32; 3],
    emitter_front: [f32; 3],
    volume_curve: Vec<(f32, f32)>,
    curve_distance_scaler: f32,
    cone: Option<OracleCone>,
) -> OracleSpatialScene {
    OracleSpatialScene {
        listener_position,
        listener_front: [0.0, 1.0, 0.0],
        listener_top: [0.0, 0.0, 1.0],
        emitter_position: [0.0, 0.0, 0.0],
        emitter_front,
        emitter_top: [0.0, 0.0, 1.0],
        volume_curve,
        curve_distance_scaler,
        cone,
    }
}

/// The voice cone of `applyILD`, in the `f32` X3DAudio takes.
fn original_voice_cone() -> OracleCone {
    let cone = EmitterCone::TFAR_VOICE;
    OracleCone {
        inner_angle: cone.inner_angle().radians() as f32,
        outer_angle: cone.outer_angle().radians() as f32,
        inner_volume: cone.inner_volume() as f32,
        outer_volume: cone.outer_volume() as f32,
    }
}

#[test]
fn the_voice_curve_matches_f3daudio_from_zero_to_twice_the_scaler() {
    let mut failures = Vec::new();
    let mut worst = 0.0_f64;
    for voice_volume in [5.0, 20.0, 40.0, 60.0] {
        let curve = our_voice_curve(voice_volume);
        let points = original_voice_curve(&curve);
        for step in 0..=400 {
            let distance = voice_volume * 2.0 * f64::from(step) / 400.0;
            // The listener stands off to one side and above, so the matrix pans the volume.
            let direction = [0.6, 0.0, 0.8];
            let position = direction.map(|component| (component * distance) as f32);
            let original = f3daudio_calculate(&scene(
                position,
                [0.0, 1.0, 0.0],
                points.clone(),
                voice_volume as f32,
                None,
            ))
            .unwrap();
            let ours = curve.gain_at(distance, voice_volume);
            let difference = (ours - f64::from(original.volume)).abs();
            worst = worst.max(difference);
            if difference.is_nan() || difference > GAIN_TOLERANCE {
                failures.push(format!(
                    "voice volume {voice_volume} at {distance:.3} m: F3DAudio {}, ours {ours}",
                    original.volume
                ));
            }
        }
    }
    println!("distance curve: worst gain difference {worst:e}");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_voice_curve_breaks_the_x3daudio_rule_that_curves_end_at_distance_one() {
    let curve = our_voice_curve(20.0);
    let original = f3daudio_calculate(&scene(
        [0.0, 10.0, 0.0],
        [0.0, 1.0, 0.0],
        original_voice_curve(&curve),
        20.0,
        None,
    ))
    .unwrap();
    assert_eq!(original.failed_parameter_checks, VOICE_CURVE_FAILED_CHECKS);
}

#[test]
fn the_voice_cone_matches_f3daudio_at_72_angles() {
    let front = [0.0_f64, 1.0, 0.0];
    let mut failures = Vec::new();
    let mut worst = 0.0_f64;
    for step in 0..72 {
        let angle = TAU * f64::from(step) / 72.0;
        // The listener two metres away, `angle` off the emitter's facing in the horizontal plane;
        // with no curve and a 100 m scaler the distance gain is 1, so the volume is the cone's.
        let offset = [2.0 * angle.sin(), 2.0 * angle.cos(), 0.0];
        let original = f3daudio_calculate(&scene(
            offset.map(|component| component as f32),
            front.map(|component| component as f32),
            Vec::new(),
            100.0,
            Some(original_voice_cone()),
        ))
        .unwrap();
        assert_eq!(original.failed_parameter_checks, 0, "step {step}");
        let ours = EmitterCone::TFAR_VOICE.gain(
            Direction::from_array(front).unwrap(),
            Direction::from_array(offset).unwrap(),
        );
        let difference = (ours - f64::from(original.volume)).abs();
        worst = worst.max(difference);
        if difference.is_nan() || difference > GAIN_TOLERANCE {
            failures.push(format!(
                "{:.1}°: F3DAudio {}, ours {ours}",
                angle.to_degrees(),
                original.volume
            ));
        }
    }
    println!("emitter cone: worst gain difference {worst:e}");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn curve_and_cone_together_match_f3daudio_in_the_apply_ild_scene() {
    let voice_volume = 20.0;
    let curve = our_voice_curve(voice_volume);
    let points = original_voice_curve(&curve);
    let mut failures = Vec::new();
    let mut worst = 0.0_f64;
    for step in 0..96 {
        // Walk the listener around the emitter on a widening spiral, the emitter facing 30° off +y.
        let bearing = TAU * f64::from(step) / 24.0;
        let distance = 0.5 + 0.3 * f64::from(step);
        let offset = [
            distance * bearing.sin(),
            distance * bearing.cos(),
            0.1 * f64::from(step % 5),
        ];
        let facing = FRAC_PI_2 / 3.0;
        let front = [facing.sin(), facing.cos(), 0.0];
        let original = f3daudio_calculate(&scene(
            offset.map(|component| component as f32),
            front.map(|component| component as f32),
            points.clone(),
            voice_volume as f32,
            Some(original_voice_cone()),
        ))
        .unwrap();
        let length = offset
            .iter()
            .map(|component| component * component)
            .sum::<f64>()
            .sqrt();
        let ours = curve.gain_at(length, voice_volume)
            * EmitterCone::TFAR_VOICE.gain(
                Direction::from_array(front).unwrap(),
                Direction::from_array(offset).unwrap(),
            );
        let difference = (ours - f64::from(original.volume)).abs();
        worst = worst.max(difference);
        if difference.is_nan() || difference > GAIN_TOLERANCE {
            failures.push(format!(
                "step {step} at {length:.2} m: F3DAudio {}, ours {ours}",
                original.volume
            ));
        }
    }
    println!("curve and cone: worst gain difference {worst:e}");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
