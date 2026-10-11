//! The one place an `f32` sample value becomes an `i16` sample.

/// Truncates `value` toward zero and saturates it to the `i16` range; `NaN` becomes 0.
///
/// Truncation keeps the originals' `static_cast<short>` rounding; saturation replaces their
/// wrap-around above full scale. Rust's float-to-integer `as` cast does exactly both.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the float-to-integer `as` cast truncates and saturates, which is the conversion wanted"
)]
pub(crate) fn saturate_to_sample(value: f32) -> i16 {
    value as i16
}
