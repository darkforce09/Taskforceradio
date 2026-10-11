//! The one place a filtered `f64` sample becomes the `f32` the effects carry.

/// Rounds `value` to the nearest `f32`; values beyond the `f32` range become infinite.
#[expect(
    clippy::cast_possible_truncation,
    reason = "narrowing a filtered sample back to the f32 block the effect processes is the intent"
)]
pub(crate) fn narrow_to_f32(value: f64) -> f32 {
    value as f32
}
