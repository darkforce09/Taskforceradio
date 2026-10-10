//! Unit tests for the flush-to-zero guard of the voice callbacks.

use super::*;

/// A value that stays denormal when halved.
fn small_denormal() -> f32 {
    f32::MIN_POSITIVE / 4.0
}

#[cfg(target_arch = "x86_64")]
#[test]
fn inside_the_guard_a_denormal_result_is_zero() {
    let guard = DenormalGuard::new();
    let halved = std::hint::black_box(f32::MIN_POSITIVE) / std::hint::black_box(4.0);
    assert_eq!(halved, 0.0);
    drop(guard);
}

#[cfg(target_arch = "x86_64")]
#[test]
fn the_guard_restores_the_register_it_found() {
    let before = read_control_register();
    let guard = DenormalGuard::new();
    assert_eq!(
        read_control_register(),
        before | FLUSH_TO_ZERO | DENORMALS_ARE_ZERO
    );
    drop(guard);
    assert_eq!(read_control_register(), before);
    let halved = std::hint::black_box(small_denormal()) / std::hint::black_box(2.0);
    assert!(halved > 0.0, "denormals work again after the guard");
}
