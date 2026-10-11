//! Direct speech in the 1.0 chain: range and distance level, vehicles, muffled speech, objects
//! with their look-back, the voice cone and the HRTF direction.

#![cfg(test)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "signal levels are measured in f64 from i16 samples and handed on as f32 gains"
)]

// `cfg(test)` puts the helpers under the test lint relief of the root `clippy.toml`.
#[cfg(test)]
mod support;

use support::{
    ALPHA, FRAMES, World, at, channel_rms, facing, in_vehicle, is_silent, rms, settled, sine,
};
use tfar_audio::dsp_filters::{FilterDesign, StereoFilter};
use tfar_audio::sample_buffer::{StereoBuffer, apply_stereo_gain};
use tfar_audio::spatial_audio::{DistanceCurve, ILD_COEFFICIENT_1_0, stereo_pan_gains};
use tfar_foundation::geometry::Position;
use tfar_plugin_logic_1_0::client_state::prelude::{
    VEHICLE_FILTER_KEY, object_interception_cutoff_hz,
};
use tfar_plugin_logic_1_0::voice_pipeline::{VoiceOutcome, volume_attenuation};

/// The level the pipeline gives direct speech of a 20 m voice at `distance`: the 1.0 voice
/// curve, as `f32`.
fn curve_level(distance: f64, should_hear: bool) -> f32 {
    let curve = DistanceCurve::tfar_voice(20.0, |metres| {
        volume_attenuation(metres, should_hear, 20.0, 1.0)
    })
    .unwrap();
    curve.gain_at(distance, 20.0) as f32
}

/// The simple-ILD gains for a sender at `sender` heard by the listener at the origin facing
/// north, times `level`.
fn fallback_gains(sender: Position, level: f32) -> (f32, f32) {
    let listener = at(0.0, 0.0, 0.0);
    let (left, right) = stereo_pan_gains(
        listener.direction_to(sender),
        facing(0.0, 1.0, 0.0).view_angle(),
        ILD_COEFFICIENT_1_0,
    );
    (left * level, right * level)
}

#[test]
fn in_range_the_level_is_the_voice_curve_at_the_distance_times_the_ild() {
    let mut world = World::new();
    let sender = at(0.0, 10.0, 0.0);
    world.client(ALPHA).position = sender;
    world.ensure(ALPHA);
    let input = sine(1000.0, 8000.0, 0);
    let (outcome, output) = world.process(ALPHA, &input);
    assert_eq!(outcome, VoiceOutcome::Processed);
    let mut expected = input.clone();
    let (left, right) = fallback_gains(sender, curve_level(10.0, true));
    apply_stereo_gain(&mut expected, left, right);
    assert_eq!(output, expected);
}

#[test]
fn the_level_falls_with_distance_as_the_curve_does() {
    let mut levels = Vec::new();
    for distance in [2.0, 8.0, 16.0, 30.0] {
        let mut world = World::new();
        world.client(ALPHA).position = at(0.0, distance, 0.0);
        levels.push(rms(&world.play_sine(ALPHA, 1000.0, 8000.0, 2)));
    }
    assert!(
        levels.windows(2).all(|pair| pair[0] > pair[1]),
        "{levels:?}"
    );
    let ratio = levels[1] / levels[0];
    let expected = f64::from(curve_level(8.0, true) / curve_level(2.0, true));
    assert!((ratio - expected).abs() < 0.01, "{ratio} vs {expected}");
}

#[test]
fn beyond_the_voice_range_and_fifteen_metres_the_sender_is_silent() {
    let mut world = World::new();
    world.client(ALPHA).position = at(0.0, 36.0, 0.0);
    let output = world.play_sine(ALPHA, 1000.0, 8000.0, 3);
    assert!(is_silent(&output));
}

#[test]
fn each_object_adds_two_metres_to_the_range_check() {
    let mut world = World::new();
    world.client(ALPHA).position = at(0.0, 30.0, 0.0);
    world.client(ALPHA).object_interception = 3;
    let output = world.play_sine(ALPHA, 300.0, 8000.0, 3);
    assert!(is_silent(&output), "30 m + 3 objects is beyond 35 m");
}

/// The 8 kHz to 300 Hz level ratio of `world`'s sender after the filters settle.
fn high_to_low_ratio(world: &mut World, high_hz: f64, low_hz: f64) -> f64 {
    let high = rms(settled(&world.play_sine(ALPHA, high_hz, 4000.0, 8), 4));
    world.client(ALPHA).effects.reset_voices();
    let low = rms(settled(&world.play_sine(ALPHA, low_hz, 4000.0, 8), 4));
    high / low
}

#[test]
fn a_sender_in_another_vehicle_is_low_passed_at_eight_kilohertz() {
    let mut world = World::new();
    world.client(ALPHA).position = at(0.0, 5.0, 0.0);
    world.client(ALPHA).vehicle = in_vehicle("2:5", 0.5, -1);
    let ratio = high_to_low_ratio(&mut world, 8000.0, 300.0);
    // Butterworth order 2 at 20000 · 0.5 / 4 = 2500 Hz: 8 kHz is about -20 dB.
    assert!(ratio < 0.15, "{ratio}");
    let mut open = World::new();
    open.client(ALPHA).position = at(0.0, 5.0, 0.0);
    let open_ratio = high_to_low_ratio(&mut open, 8000.0, 300.0);
    assert!(open_ratio > 0.9, "{open_ratio}");
}

#[test]
fn a_sender_in_another_vehicle_is_quieter_by_the_isolation() {
    let mut world = World::new();
    world.client(ALPHA).position = at(0.0, 5.0, 0.0);
    world.client(ALPHA).vehicle = in_vehicle("2:5", 0.5, -1);
    let muffled = rms(settled(&world.play_sine(ALPHA, 300.0, 4000.0, 8), 4));
    let mut open = World::new();
    open.client(ALPHA).position = at(0.0, 5.0, 0.0);
    let clear = rms(settled(&open.play_sine(ALPHA, 300.0, 4000.0, 8), 4));
    let expected = volume_attenuation(5.0, true, 20.0, 0.5) * 0.5_f64.powf(1.2);
    let ratio = muffled / clear;
    assert!((ratio - expected).abs() < 0.03, "{ratio} vs {expected}");
    assert!(
        world
            .client(ALPHA)
            .effects
            .vehicle_filter_mut(VEHICLE_FILTER_KEY, 0.5)
            .is_some()
    );
}

#[test]
fn two_players_in_the_same_vehicle_hear_each_other_clearly() {
    let mut world = World::new();
    world.client(ALPHA).position = at(0.0, 2.0, 0.0);
    world.client(ALPHA).vehicle = in_vehicle("2:5", 0.5, -1);
    world.session.clients.own_mut().unwrap().vehicle = in_vehicle("2:5", 0.5, -1);
    let ratio = high_to_low_ratio(&mut world, 8000.0, 300.0);
    assert!(ratio > 0.9, "{ratio}");
}

#[test]
fn a_listener_who_cannot_speak_hears_a_hundred_hertz_low_pass() {
    let mut world = World::new();
    world.client(ALPHA).position = at(0.0, 2.0, 0.0);
    world.session.clients.own_mut().unwrap().can_speak = false;
    let ratio = high_to_low_ratio(&mut world, 1000.0, 50.0);
    // Butterworth order 4 at 100 Hz: 1 kHz is about -80 dB.
    assert!(ratio < 0.01, "{ratio}");
    let low = rms(settled(&world.play_sine(ALPHA, 50.0, 500.0, 8), 4));
    assert!(low > 50.0, "the muffled voice is still heard: {low}");
}

#[test]
fn objects_between_the_two_low_pass_the_voice() {
    let mut world = World::new();
    world.client(ALPHA).position = at(0.0, 5.0, 0.0);
    world.client(ALPHA).object_interception = 2;
    // 2000 - 2 · 400 = 1200 Hz.
    let ratio = high_to_low_ratio(&mut world, 8000.0, 200.0);
    assert!(ratio < 0.05, "{ratio}");
}

#[test]
fn the_first_frame_behind_objects_has_no_look_back_and_is_filtered_once() {
    let mut world = World::new();
    let sender = at(0.0, 5.0, 0.0);
    world.client(ALPHA).position = sender;
    world.client(ALPHA).object_interception = 2;
    world.ensure(ALPHA);
    let input = sine(730.0, 8000.0, 0);
    let (_, output) = world.process(ALPHA, &input);

    let cutoff_hz = object_interception_cutoff_hz(2, 400.0);
    let design = FilterDesign::ButterworthLowPass {
        order: 2,
        cutoff_hz,
    };
    let mut reference = StereoFilter::new(design, 48_000.0).unwrap();
    let mut expected = input.clone();
    reference.process_stereo_i16(&mut expected, 1.0);
    let (left, right) = fallback_gains(sender, curve_level(5.0, true));
    apply_stereo_gain(&mut expected, left, right);
    assert_eq!(output, expected);
}

#[test]
fn a_filter_switched_to_by_a_new_object_count_is_warmed_with_the_previous_frame() {
    let mut world = World::new();
    let sender = at(0.0, 5.0, 0.0);
    world.client(ALPHA).position = sender;
    for count in [3, 2] {
        world.client(ALPHA).object_interception = count;
        world.ensure(ALPHA);
    }
    let first = sine(730.0, 8000.0, 0);
    let second = sine(730.0, 8000.0, 1);
    assert_ne!(first, second, "a frame of 7.3 cycles differs from the next");
    world.process(ALPHA, &first);
    world.client(ALPHA).object_interception = 3;
    let (_, output) = world.process(ALPHA, &second);
    assert_eq!(
        world.client(ALPHA).effects.object_interception_buffer_mut(),
        &second,
        "the current frame is kept as the next look-back"
    );

    // The filter for three objects has not run yet: the look-back warms it with the first frame.
    let design = FilterDesign::ButterworthLowPass {
        order: 2,
        cutoff_hz: object_interception_cutoff_hz(3, 400.0),
    };
    let mut reference = StereoFilter::new(design, 48_000.0).unwrap();
    reference.process_stereo_i16(&mut first.clone(), 1.0);
    let mut expected = second.clone();
    reference.process_stereo_i16(&mut expected, 1.0);
    let (left, right) = fallback_gains(sender, curve_level(5.0, true));
    apply_stereo_gain(&mut expected, left, right);
    assert_eq!(output, expected);
}

/// The level of a sender 10 m north whose view is `view`, with the voice cone on.
fn cone_level(view: [f64; 3]) -> f64 {
    let mut world = World::new();
    world.set_bool("voiceCone", true);
    world.client(ALPHA).position = at(0.0, 10.0, 0.0);
    world.client(ALPHA).view_direction = facing(view[0], view[1], view[2]);
    rms(&world.play_sine(ALPHA, 1000.0, 4000.0, 2))
}

#[test]
fn a_sender_facing_the_listener_is_one_point_eight_times_as_loud_as_one_facing_away() {
    let front = cone_level([0.0, -1.0, 0.0]);
    let back = cone_level([0.0, 1.0, 0.0]);
    let ratio = front / back;
    assert!((ratio - 1.8).abs() < 0.01, "{ratio}");
}

#[test]
fn a_sender_facing_sideways_is_heard_at_the_outer_volume() {
    let side = cone_level([1.0, 0.0, 0.0]);
    let back = cone_level([0.0, 1.0, 0.0]);
    assert!((side / back - 1.0).abs() < 0.01, "{}", side / back);
}

#[test]
fn a_sender_to_the_right_is_louder_in_the_right_ear_through_the_hrtf() {
    let mut world = World::with_hrtf();
    world.client(ALPHA).position = at(10.0, 0.0, 0.0);
    let output = world.play_sine(ALPHA, 1000.0, 4000.0, 12);
    let late = settled(&output, 4);
    let (left, right) = (channel_rms(late, 0), channel_rms(late, 1));
    assert!(right > 2.0 * left, "left {left} right {right}");
    assert!(
        world
            .client(ALPHA)
            .effects
            .hrtf_source_mut(tfar_plugin_logic_1_0::client_state::prelude::VOICE_HRTF_KEY)
            .is_some()
    );
}

#[test]
fn straight_ahead_the_hrtf_with_its_makeup_keeps_about_the_input_level() {
    let mut world = World::with_hrtf();
    world.client(ALPHA).position = at(0.0, 1.0, 0.0);
    let output = world.play_sine(ALPHA, 1000.0, 4000.0, 12);
    let level = rms(settled(&output, 4));
    let input = 4000.0 / 2.0_f64.sqrt();
    let decibels = 20.0 * (level / input).log10();
    assert!(decibels.abs() < 3.0, "{decibels} dB");
}

#[test]
fn the_hrtf_output_lags_by_one_render_block() {
    let mut world = World::with_hrtf();
    world.client(ALPHA).position = at(0.0, 1.0, 0.0);
    let output = world.play_sine(ALPHA, 1000.0, 4000.0, 2);
    assert_eq!(output[0].frames(), FRAMES);
    assert!(
        is_silent(&output[..1]),
        "the first 480 frames fill the block"
    );
    assert!(!is_silent(&output[1..]));
}

#[test]
fn without_the_dsp_objects_the_voice_still_passes_at_its_level() {
    let mut world = World::new();
    let sender = at(0.0, 5.0, 0.0);
    world.client(ALPHA).position = sender;
    let input = sine(730.0, 8000.0, 0);
    let (outcome, output) = world.process(ALPHA, &input);
    assert_eq!(outcome, VoiceOutcome::Processed);
    let mut expected: StereoBuffer = input.clone();
    let (left, right) = fallback_gains(sender, curve_level(5.0, true));
    apply_stereo_gain(&mut expected, left, right);
    assert_eq!(output, expected);
}
