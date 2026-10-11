//! Unit tests for `HrtfSource` over the real IRCAM sphere: the ear-energy balance of each
//! direction, silence, the exact latency across variable frame sizes, stable buffers, `reset` and
//! the reversal guard, and the idle restart.

use std::time::Duration;

use tfar_geometry::{Angle, Position};

use super::*;
use crate::real_sphere::real_library;
use crate::render_timing::ADDED_LATENCY_FRAMES;

/// TeamSpeak's frame sizes, plus an odd one and a single sample.
const FRAME_SIZES: [usize; 4] = [480, 960, 441, 1];
/// Two seconds at 48 kHz.
const TWO_SECONDS: usize = 96_000;

/// Deterministic white noise at a quarter of full scale.
fn noise(frames: usize, seed: u32) -> Vec<i16> {
    let mut state = seed;
    (0..frames)
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            ((state >> 16) as u16).cast_signed() / 4
        })
        .collect()
}

/// The energy of each ear and of each ear's first difference (a high-frequency emphasis) after
/// one second of noise from `direction`, skipping the first 100 ms.
struct EarEnergy {
    left: f64,
    right: f64,
    left_high: f64,
    right_high: f64,
}

impl EarEnergy {
    fn of(direction: ListenerRelativeDirection) -> Self {
        let library = real_library();
        let mut source = HrtfSource::new(&library);
        let input = noise(48_000, 0x1234_5678);
        let mut output = StereoBuffer::with_frames(input.len());
        source.process_mono(&input, &mut output, direction, 1.0);
        let mut energy = Self {
            left: 0.0,
            right: 0.0,
            left_high: 0.0,
            right_high: 0.0,
        };
        let frames = output.as_slice().as_chunks::<2>().0;
        for pair in frames[4_800..].windows(2) {
            let [previous, current] = [pair[0].map(f64::from), pair[1].map(f64::from)];
            energy.left += current[0].powi(2);
            energy.right += current[1].powi(2);
            energy.left_high += (current[0] - previous[0]).powi(2);
            energy.right_high += (current[1] - previous[1]).powi(2);
        }
        energy
    }

    /// Left minus right, in dB.
    fn left_over_right_db(&self) -> f64 {
        10.0 * (self.left / self.right).log10()
    }

    /// Both ears together.
    fn total(&self) -> f64 {
        self.left + self.right
    }

    /// The share of high-frequency energy, both ears together.
    fn high_share(&self) -> f64 {
        (self.left_high + self.right_high) / self.total()
    }
}

fn axes(right: f64, forward: f64, up: f64) -> ListenerRelativeDirection {
    ListenerRelativeDirection::from_listener_axes(right, forward, up)
}

#[test]
fn a_source_straight_ahead_reaches_both_ears_equally() {
    let balance = EarEnergy::of(axes(0.0, 1.0, 0.0)).left_over_right_db();
    assert!(balance.abs() < 2.0, "{balance} dB");
}

#[test]
fn a_source_ninety_degrees_right_is_louder_in_the_right_ear() {
    // Facing east, a source to the south is on the right.
    let direction = ListenerRelativeDirection::from_arma(
        Position::from_array([100.0, 200.0, 5.0]).unwrap(),
        Angle::from_degrees(90.0),
        Position::from_array([100.0, 190.0, 5.0]).unwrap(),
    );
    let balance = EarEnergy::of(direction).left_over_right_db();
    assert!(balance < -10.0, "{balance} dB");
}

#[test]
fn a_source_ninety_degrees_left_is_louder_in_the_left_ear() {
    let balance = EarEnergy::of(axes(-1.0, 0.0, 0.0)).left_over_right_db();
    assert!(balance > 10.0, "{balance} dB");
}

#[test]
fn a_source_behind_reaches_both_ears_equally() {
    let balance = EarEnergy::of(axes(0.0, -1.0, 0.0)).left_over_right_db();
    assert!(balance.abs() < 2.0, "{balance} dB");
}

#[test]
fn a_source_above_stays_roughly_symmetric() {
    let balance = EarEnergy::of(axes(0.0, 0.0, 1.0)).left_over_right_db();
    assert!(balance.abs() < 2.0, "{balance} dB");
}

#[test]
fn a_source_ahead_is_louder_and_brighter_than_the_same_source_behind() {
    // The pinnae shade sound from behind, above all its high frequencies; this pins the sign of
    // the sphere's forward axis, which the left-right tests cannot see.
    let ahead = EarEnergy::of(axes(0.0, 1.0, 0.0));
    let behind = EarEnergy::of(axes(0.0, -1.0, 0.0));
    let louder_db = 10.0 * (ahead.total() / behind.total()).log10();
    assert!(louder_db > 3.0, "{louder_db} dB");
    assert!(ahead.high_share() > behind.high_share() * 1.5);
}

#[test]
fn zero_input_gives_zero_output() {
    let library = real_library();
    let mut source = HrtfSource::new(&library);
    let silence = vec![0_i16; 960];
    let mut output = StereoBuffer::with_frames(960);
    for step in 0..100 {
        let direction = axes(f64::from(step).sin(), f64::from(step).cos(), 0.2);
        source.process_mono(&silence, &mut output, direction, 1.0);
        assert!(output.as_slice().iter().all(|sample| *sample == 0));
    }
}

/// Stereo noise with different left and right channels, frame by frame.
fn stereo_noise(frames: usize) -> Vec<[i16; 2]> {
    noise(frames, 7)
        .into_iter()
        .zip(noise(frames, 99))
        .map(|(left, right)| [left, right])
        .collect()
}

/// What the `hrtf` processor gives when driven directly in whole blocks with the mono mean of
/// `input`, a fixed `direction` and `gain`.
fn rendered_directly(
    input: &[[i16; 2]],
    direction: ListenerRelativeDirection,
    gain: f32,
) -> Vec<[i16; 2]> {
    let library = real_library();
    let mut processor = library.new_processor();
    let tail_length = library.hrir_length() - 1;
    let mut left_tail = vec![0.0; tail_length];
    let mut right_tail = vec![0.0; tail_length];
    let vector = direction.sphere_sampling_vector();
    let mut rendered = Vec::with_capacity(input.len());
    for block in input.as_chunks::<RENDER_BLOCK_FRAMES>().0 {
        let mono: Vec<f32> = block
            .iter()
            .map(|[left, right]| f32::midpoint(sample_to_float(*left), sample_to_float(*right)))
            .collect();
        let mut output = vec![(0.0, 0.0); RENDER_BLOCK_FRAMES];
        processor.process_samples(HrtfContext {
            source: &mono,
            output: &mut output,
            new_sample_vector: vector,
            prev_sample_vector: vector,
            prev_left_samples: &mut left_tail,
            prev_right_samples: &mut right_tail,
            new_distance_gain: gain,
            prev_distance_gain: gain,
        });
        rendered.extend(
            output
                .iter()
                .map(|(left, right)| [float_to_sample(*left), float_to_sample(*right)]),
        );
    }
    rendered
}

#[test]
fn variable_frame_sizes_give_continuous_output_with_exactly_the_documented_latency() {
    let direction = axes(0.7, 0.7, 0.1);
    let gain = 0.8;
    let input = stereo_noise(TWO_SECONDS);
    let expected = rendered_directly(&input, direction, gain);

    let library = real_library();
    let mut source = HrtfSource::new(&library);
    let mut frame_in = StereoBuffer::with_frames(960);
    let mut frame_out = StereoBuffer::with_frames(960);
    let mut output: Vec<[i16; 2]> = Vec::with_capacity(TWO_SECONDS);
    let mut consumed = 0;
    for frame_size in FRAME_SIZES.iter().cycle() {
        if consumed == TWO_SECONDS {
            break;
        }
        let frame_size = (*frame_size).min(TWO_SECONDS - consumed);
        frame_in.set_frames(frame_size);
        for (target, source_frame) in frame_in
            .as_mut_slice()
            .as_chunks_mut::<2>()
            .0
            .iter_mut()
            .zip(&input[consumed..consumed + frame_size])
        {
            *target = *source_frame;
        }
        source.process(&frame_in, &mut frame_out, direction, gain);
        assert_eq!(frame_out.frames(), frame_size);
        output.extend_from_slice(frame_out.as_slice().as_chunks::<2>().0);
        consumed += frame_size;
    }

    assert_eq!(output.len(), TWO_SECONDS);
    assert!(
        output[..ADDED_LATENCY_FRAMES]
            .iter()
            .all(|frame| *frame == [0, 0])
    );
    for (index, frame) in output.iter().enumerate().skip(ADDED_LATENCY_FRAMES) {
        assert_eq!(
            *frame,
            expected[index - ADDED_LATENCY_FRAMES],
            "output frame {index}"
        );
    }
    assert!(output.iter().filter(|frame| **frame != [0, 0]).count() > TWO_SECONDS / 2);
}

#[test]
fn process_mono_matches_process_with_both_channels_equal() {
    let library = real_library();
    let mut stereo_source = HrtfSource::new(&library);
    let mut mono_source = HrtfSource::new(&library);
    let mono = noise(2_000, 3);
    let mut stereo_in = StereoBuffer::with_frames(0);
    tfar_sample_buffer::mono_to_stereo(&mono, &mut stereo_in);
    let mut stereo_out = StereoBuffer::with_frames(0);
    let mut mono_out = StereoBuffer::with_frames(0);
    let direction = axes(-0.3, 0.5, 0.4);
    stereo_source.process(&stereo_in, &mut stereo_out, direction, 1.0);
    mono_source.process_mono(&mono, &mut mono_out, direction, 1.0);
    assert_eq!(stereo_out, mono_out);
}

#[test]
fn no_buffer_reallocates_after_warm_up() {
    let library = real_library();
    let mut source = HrtfSource::new(&library);
    let mut frame_in = StereoBuffer::with_frames(960);
    let mut frame_out = StereoBuffer::with_frames(960);
    let fingerprint = |source: &HrtfSource, frame_out: &StereoBuffer| {
        [
            source.renderer.previous_left_tail.as_ptr().addr(),
            source.renderer.previous_left_tail.capacity(),
            source.renderer.previous_right_tail.as_ptr().addr(),
            source.renderer.previous_right_tail.capacity(),
            frame_out.as_slice().as_ptr().addr(),
            frame_out.capacity_frames(),
        ]
    };
    let tail_length = library.hrir_length() - 1;
    assert_eq!(source.renderer.previous_left_tail.len(), tail_length);
    let warm_up_input = noise(960, 5);
    source.process_mono(&warm_up_input, &mut frame_out, axes(0.0, 1.0, 0.0), 1.0);
    let after_warm_up = fingerprint(&source, &frame_out);

    let input = stereo_noise(TWO_SECONDS);
    let mut angle = 0.0_f64;
    let mut consumed = 0;
    for (step, frame_size) in FRAME_SIZES.iter().cycle().enumerate() {
        if consumed + frame_size > TWO_SECONDS {
            break;
        }
        frame_in.set_frames(*frame_size);
        for (target, source_frame) in frame_in
            .as_mut_slice()
            .as_chunks_mut::<2>()
            .0
            .iter_mut()
            .zip(&input[consumed..])
        {
            *target = *source_frame;
        }
        angle += 0.05;
        source.process(
            &frame_in,
            &mut frame_out,
            axes(angle.sin(), angle.cos(), 0.0),
            0.5,
        );
        assert_eq!(
            fingerprint(&source, &frame_out),
            after_warm_up,
            "frame {step}"
        );
        consumed += frame_size;
    }
    source.reset();
    assert_eq!(fingerprint(&source, &frame_out), after_warm_up);
    assert_eq!(source.renderer.previous_left_tail.len(), tail_length);
}

#[test]
fn reset_silences_the_source_and_starts_a_new_first_fill() {
    let library = real_library();
    let mut source = HrtfSource::new(&library);
    let mut output = StereoBuffer::with_frames(0);
    source.process_mono(&noise(4_800, 11), &mut output, axes(1.0, 0.0, 0.0), 1.0);
    assert!(output.as_slice().iter().any(|sample| *sample != 0));
    source.reset();
    source.process_mono(&vec![0; 4_800], &mut output, axes(1.0, 0.0, 0.0), 1.0);
    assert!(output.as_slice().iter().all(|sample| *sample == 0));
    source.process_mono(
        &noise(ADDED_LATENCY_FRAMES, 12),
        &mut output,
        axes(1.0, 0.0, 0.0),
        1.0,
    );
    assert!(output.as_slice().iter().all(|sample| *sample == 0));
}

/// A source that rendered noise, its last input moved `idle` into the past.
fn source_idle_for(library: &HrirLibrary, idle: Duration) -> HrtfSource {
    let mut source = HrtfSource::new(library);
    let mut output = StereoBuffer::with_frames(0);
    source.process_mono(&noise(4_800, 21), &mut output, axes(1.0, 0.0, 0.0), 1.0);
    source.last_input = Instant::now().checked_sub(idle);
    source
}

#[test]
fn a_frame_after_a_long_gap_starts_from_silence() {
    let library = real_library();
    let mut source = source_idle_for(&library, IDLE_RESET_AFTER * 2);
    let mut output = StereoBuffer::with_frames(0);
    source.process_mono(
        &noise(ADDED_LATENCY_FRAMES, 22),
        &mut output,
        axes(1.0, 0.0, 0.0),
        1.0,
    );
    assert!(output.as_slice().iter().all(|sample| *sample == 0));
}

#[test]
fn a_frame_after_a_short_gap_continues_the_utterance() {
    let library = real_library();
    let mut source = source_idle_for(&library, IDLE_RESET_AFTER / 2);
    let mut output = StereoBuffer::with_frames(0);
    source.process_mono(
        &vec![0; ADDED_LATENCY_FRAMES],
        &mut output,
        axes(1.0, 0.0, 0.0),
        1.0,
    );
    assert!(
        output.as_slice().iter().any(|sample| *sample != 0),
        "the pending block of the noise still plays"
    );
}

#[test]
fn a_non_finite_or_negative_gain_is_silence() {
    let library = real_library();
    for gain in [f32::NAN, f32::INFINITY, -1.0, 0.0] {
        let mut source = HrtfSource::new(&library);
        let mut output = StereoBuffer::with_frames(0);
        source.process_mono(&noise(2_400, 13), &mut output, axes(0.0, 1.0, 0.0), gain);
        assert!(
            output.as_slice().iter().all(|sample| *sample == 0),
            "gain {gain}"
        );
    }
}

#[test]
fn a_block_glides_from_the_previous_direction_but_snaps_across_a_reversal() {
    let ahead = axes(0.0, 1.0, 0.0).sphere_sampling_vector();
    let right = axes(1.0, 0.0, 0.0).sphere_sampling_vector();
    let behind = axes(0.0, -1.0, 0.0).sphere_sampling_vector();
    let almost_behind = axes(0.4, -1.0, 0.0).sphere_sampling_vector();
    assert_eq!(block_start_vector(None, ahead), ahead);
    assert_eq!(block_start_vector(Some(right), ahead), right);
    assert_eq!(block_start_vector(Some(behind), ahead), ahead);
    assert_eq!(block_start_vector(Some(almost_behind), ahead), ahead);
    // 150 degrees apart still glides: the halfway vector keeps a quarter of unit length.
    let wide = axes(0.5, -(0.75_f64.sqrt()), 0.0).sphere_sampling_vector();
    assert_eq!(block_start_vector(Some(wide), ahead), wide);
}

/// `count` directions spread evenly over the whole sphere (a Fibonacci lattice).
fn directions_over_the_sphere(count: u32) -> Vec<ListenerRelativeDirection> {
    let golden_angle = core::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    (0..count)
        .map(|index| {
            let up = 1.0 - 2.0 * (f64::from(index) + 0.5) / f64::from(count);
            let radius = (1.0 - up * up).sqrt();
            let azimuth = golden_angle * f64::from(index);
            axes(radius * azimuth.cos(), radius * azimuth.sin(), up)
        })
        .collect()
}

#[test]
fn every_direction_and_every_jump_between_them_renders_without_panicking() {
    let library = real_library();
    let mut source = HrtfSource::new(&library);
    let block = noise(RENDER_BLOCK_FRAMES, 17);
    let mut output = StereoBuffer::with_frames(RENDER_BLOCK_FRAMES);
    let directions = directions_over_the_sphere(1_500);
    for (index, direction) in directions.iter().enumerate() {
        source.process_mono(&block, &mut output, *direction, 1.0);
        // Jump to the opposite side of the sphere and back: the reversal guard's case.
        let opposite = axes(-direction.right(), -direction.forward(), -direction.up());
        source.process_mono(&block, &mut output, opposite, 1.0);
        let next = directions[(index * 7 + 3) % directions.len()];
        source.process_mono(&block, &mut output, next, 1.0);
    }
    assert!(output.as_slice().iter().any(|sample| *sample != 0));
}
