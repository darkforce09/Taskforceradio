//! The one-line statement of what `tfar_golden_vectors` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_golden_vectors` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_golden_vectors: the test oracle for the audio crates: builds \
     the original DSPFilters and SimpleComp code from reference/ and FAudio's F3DAudio with cc, \
     and runs them live beside the Rust filters, compressor, distance curve and emitter cone, \
     which must match them within tolerance.";
