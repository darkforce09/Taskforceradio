//! Unit tests for `MonoMixBuffer` in `tfar_radio_effects_1_0`.

use super::*;

#[test]
fn a_block_inside_the_capacity_reuses_the_allocation() {
    let mut buffer = MonoMixBuffer::with_frames(960);
    assert!(buffer.capacity_frames() >= 960);
    let first = buffer.block(960).as_ptr();
    let second = buffer.block(480).as_ptr();
    let third = buffer.block(960).as_ptr();
    assert_eq!(first, second);
    assert_eq!(first, third);
    assert_eq!(buffer.block(480).len(), 480);
}

#[test]
fn a_larger_block_grows_the_buffer() {
    let mut buffer = MonoMixBuffer::default();
    assert_eq!(buffer.block(256).len(), 256);
    assert!(buffer.capacity_frames() >= 256);
}
