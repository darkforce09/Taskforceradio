//! Unit tests for additive mixing in `tfar_sample_buffer`.

use super::*;

#[test]
fn mixing_adds_sample_by_sample() {
    let mut dest = [1, -2, 300];
    mix_into_additive(&mut dest, &[10, 20, -30]);
    assert_eq!(dest, [11, 18, 270]);
}

#[test]
fn mixing_saturates_at_both_limits() {
    let mut dest = [30_000, -30_000];
    mix_into_additive(&mut dest, &[10_000, -10_000]);
    assert_eq!(dest, [i16::MAX, i16::MIN]);
}

#[test]
fn mixing_a_longer_source_touches_only_the_destination_length() {
    let mut dest = [1, 2];
    mix_into_additive(&mut dest, &[10, 20, 30, 40]);
    assert_eq!(dest, [11, 22]);
}

#[test]
fn mixing_a_shorter_source_leaves_the_rest_of_the_destination_untouched() {
    let mut dest = [1, 2, 3, 4];
    mix_into_additive(&mut dest, &[10]);
    assert_eq!(dest, [11, 2, 3, 4]);
}
