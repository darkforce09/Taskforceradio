//! Additive mixing of one `i16` block into another.

#[cfg(test)]
#[path = "tests/mixing.rs"]
mod tests;

/// Adds every sample of `src` to the sample of `dest` at the same index, saturating at the `i16`
/// limits. Only the first `min(dest.len(), src.len())` samples are touched; nothing allocates.
pub fn mix_into_additive(dest: &mut [i16], src: &[i16]) {
    for (dest_sample, src_sample) in dest.iter_mut().zip(src) {
        *dest_sample = dest_sample.saturating_add(*src_sample);
    }
}
