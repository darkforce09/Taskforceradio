//! `is_true`: the original's reading of a boolean field.

#[cfg(test)]
#[path = "tests/arma_truth.rs"]
mod tests;

/// Whether `text` is true as `helpers::isTrue` reads it
/// (`reference/tfar_1_0/ts/src/helpers.cpp:296-300`): exactly `true` or exactly `1`. Every other
/// text, `TRUE` and `0` included, is false; the SQF side writes either `true`/`false` or a number
/// (`_useDd` is `0` when unused).
pub(crate) fn is_true(text: &str) -> bool {
    text == "true" || text == "1"
}
