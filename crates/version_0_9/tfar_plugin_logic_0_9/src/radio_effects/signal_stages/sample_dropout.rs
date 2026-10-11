//! `SampleDropout`: the random sample drops of the underwater effect
//! (`UnderWaterRadioEffect::process`, `reference/tfar_0_9_12/ts/src/RadioEffect.h:66`).
//!
//! The original draws from the C library's global, unseeded `rand()`; here each effect owns a
//! `fastrand` generator seeded by its creator, so a run is reproducible and effects never share a
//! sequence. The draw keeps the range of the Windows `rand()` the original ran with.

#[cfg(test)]
#[path = "tests/sample_dropout.rs"]
mod tests;

use fastrand::Rng;

/// The largest value of the Windows C library's `rand()`: draws lie in `0..=RAND_MAX`.
const RAND_MAX: i32 = 32_767;

/// [`RAND_MAX`] as the `f32` the original multiplies the error level by.
const RAND_MAX_AS_FLOAT: f32 = 32_767.0;

/// Decides per sample whether it drops out: a draw in `0..=32767` below
/// `trunc(32767 · error)` drops it, so the drop probability is about the error level.
#[derive(Debug, Clone)]
pub(crate) struct SampleDropout {
    generator: Rng,
    drop_below: i32,
}

impl SampleDropout {
    /// A dropout that never drops until an error level is set, drawing from a generator seeded
    /// with `seed`.
    pub(crate) fn new(seed: u64) -> Self {
        Self {
            generator: Rng::with_seed(seed),
            drop_below: 0,
        }
    }

    /// Sets the drop threshold from `error_level`: `trunc(32767 · error)`, saturating, `NaN`
    /// giving 0 (never drop).
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the original truncates the product to an int; the cast truncates and saturates"
    )]
    pub(crate) fn set_error_level(&mut self, error_level: f32) {
        self.drop_below = (RAND_MAX_AS_FLOAT * error_level) as i32;
    }

    /// Draws once and tells whether this sample drops out.
    pub(crate) fn should_drop(&mut self) -> bool {
        self.generator.i32(0..=RAND_MAX) < self.drop_below
    }
}
