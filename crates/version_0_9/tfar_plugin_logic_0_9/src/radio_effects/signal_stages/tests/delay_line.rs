//! Unit tests for `DelayLine` in `tfar_plugin_logic_0_9::radio_effects`.

use super::*;

#[test]
fn an_impulse_comes_back_exactly_2399_samples_later() {
    let mut delay_line = DelayLine::new();
    let output: Vec<f32> = (0..3 * DELAY_LINE_SLOTS)
        .map(|index| delay_line.delay(if index == 0 { 1.0 } else { 0.0 }))
        .collect();
    let sounding: Vec<usize> = output
        .iter()
        .enumerate()
        .filter(|(_, sample)| **sample != 0.0)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(sounding, vec![2399]);
    assert_eq!(output[2399], 1.0);
}

#[test]
fn a_fresh_delay_line_plays_silence() {
    let mut delay_line = DelayLine::new();
    assert!((0..DELAY_LINE_SLOTS - 1).all(|_| delay_line.delay(0.5) == 0.0));
    assert_eq!(delay_line.delay(0.5), 0.5);
}
