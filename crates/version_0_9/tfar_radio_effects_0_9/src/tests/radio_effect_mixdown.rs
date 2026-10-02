//! Unit tests for `process_radio_effect` in `tfar_radio_effects_0_9`.

use super::*;
use crate::effects::{LongRangeRadioEffect, PersonalRadioEffect, UnderWaterRadioEffect};

/// An effect that leaves the mono block as it is and remembers the block length it saw.
#[derive(Default)]
struct PassThroughEffect {
    block_lengths: Vec<usize>,
}

impl RadioEffect for PassThroughEffect {
    fn set_error_level(&mut self, _distance_error: f64) {}

    fn process(&mut self, block: &mut [f32]) {
        self.block_lengths.push(block.len());
    }
}

/// A stereo buffer holding `frames`.
fn stereo(frames: &[[i16; 2]]) -> StereoBuffer {
    let mut buffer = StereoBuffer::with_frames(frames.len());
    let (chunks, _) = buffer.as_mut_slice().as_chunks_mut::<2>();
    chunks.copy_from_slice(frames);
    buffer
}

/// The frames of `buffer` after a pass-through run with `gain` and `stereo_mode`.
fn pass_through(frames: &[[i16; 2]], gain: f32, stereo_mode: StereoMode) -> Vec<[i16; 2]> {
    let mut buffer = stereo(frames);
    let mut mono_mix = MonoMixBuffer::with_frames(frames.len());
    process_radio_effect(
        &mut PassThroughEffect::default(),
        &mut buffer,
        gain,
        stereo_mode,
        &mut mono_mix,
    );
    buffer.as_slice().as_chunks::<2>().0.to_vec()
}

#[test]
fn both_ears_get_the_mono_mix() {
    assert_eq!(
        pass_through(&[[1000, 3000]], 1.0, StereoMode::Both),
        vec![[1999, 1999]]
    );
}

#[test]
fn one_ear_gets_the_mono_mix_with_half_again_the_gain_and_the_other_is_silent() {
    assert_eq!(
        pass_through(&[[1000, 3000]], 1.0, StereoMode::Left),
        vec![[2999, 0]]
    );
    assert_eq!(
        pass_through(&[[1000, 3000]], 1.0, StereoMode::Right),
        vec![[0, 2999]]
    );
}

#[test]
fn the_mixdown_and_the_output_truncate_toward_zero() {
    assert_eq!(
        pass_through(&[[-3001, 0]], 1.0, StereoMode::Both),
        vec![[-1499, -1499]]
    );
    assert_eq!(pass_through(&[[1, 0]], 1.0, StereoMode::Both), vec![[0, 0]]);
}

#[test]
fn a_sample_beyond_full_scale_clips_to_the_i16_limits() {
    assert_eq!(
        pass_through(&[[30000, 30000]], 4.0, StereoMode::Both),
        vec![[32767, 32767]]
    );
    assert_eq!(
        pass_through(&[[-30000, -30000]], 4.0, StereoMode::Left),
        vec![[-32768, 0]]
    );
}

#[test]
fn the_effect_runs_once_over_the_whole_block() {
    let mut effect = PassThroughEffect::default();
    let mut buffer = StereoBuffer::with_frames(960);
    let mut mono_mix = MonoMixBuffer::with_frames(960);
    process_radio_effect(
        &mut effect,
        &mut buffer,
        1.0,
        StereoMode::Both,
        &mut mono_mix,
    );
    assert_eq!(effect.block_lengths, vec![960]);
    assert_eq!(mono_mix.capacity_frames(), 960);
}

#[test]
fn silence_in_gives_silence_out_for_every_effect_and_mode() {
    let mut effects: Vec<Box<dyn RadioEffect>> = vec![
        Box::new(LongRangeRadioEffect::new().unwrap()),
        Box::new(PersonalRadioEffect::new().unwrap()),
        Box::new(UnderWaterRadioEffect::new(5).unwrap()),
    ];
    let mut mono_mix = MonoMixBuffer::with_frames(480);
    for effect in &mut effects {
        effect.set_error_level(1.0);
        for stereo_mode in [StereoMode::Both, StereoMode::Left, StereoMode::Right] {
            let mut buffer = StereoBuffer::with_frames(480);
            process_radio_effect(
                effect.as_mut(),
                &mut buffer,
                0.35,
                stereo_mode,
                &mut mono_mix,
            );
            assert!(buffer.as_slice().iter().all(|sample| *sample == 0));
        }
    }
}
