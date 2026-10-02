//! `split_fields`: splitting a message into its fields the way `std::getline` does.

#[cfg(test)]
#[path = "tests/field_split.rs"]
mod tests;

/// `input` split at every `separator`, with the semantics of the `std::getline` loop in
/// `reference/tfar_0_9_12/ts/src/plugin.cpp` (`split`):
///
/// - an empty input gives no field at all;
/// - an empty field between two separators, or before the first, is kept;
/// - one empty field after the last separator is dropped (`"a\t"` is one field, `"a\t\t"` two).
pub fn split_fields(input: &str, separator: char) -> Vec<&str> {
    let mut fields: Vec<&str> = input.split(separator).collect();
    if fields.last().is_some_and(|last| last.is_empty()) {
        fields.pop();
    }
    fields
}

/// `input` split at every `separator`, keeping every field, a trailing empty one included: the
/// semantics of `helpers::split` in `reference/tfar_1_0/ts/src/helpers.cpp`. An empty input gives
/// one empty field.
pub(crate) fn split_keeping_every_field(input: &str, separator: char) -> Vec<&str> {
    input.split(separator).collect()
}
