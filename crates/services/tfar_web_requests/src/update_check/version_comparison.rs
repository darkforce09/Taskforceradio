//! Whether a published version is newer than the plugin's: the body must be a dotted version, and
//! the comparison runs component by component with `version-compare`.
//!
//! Each component compares as a number and every component counts, so `0.9.12` is newer than
//! `0.9.3` and `1.0.0.1` is newer than `1.0`.

#[cfg(test)]
#[path = "tests/version_comparison.rs"]
mod tests;

use version_compare::Cmp;

/// Whether `text` is a dotted version: one or more groups of ASCII digits joined by single dots,
/// such as `0.9.12` or `1.0.0.1`. Surrounding whitespace is not part of the version.
pub fn is_version_string(text: &str) -> bool {
    !text.is_empty()
        && text.split('.').all(|component| {
            !component.is_empty() && component.bytes().all(|byte| byte.is_ascii_digit())
        })
}

/// Whether `latest` (the published version, as fetched, before trimming) is newer than
/// `current` (the plugin's own version).
///
/// A `latest` that does not trim to a dotted version, an empty body or an HTML error page for
/// instance, is never newer; so is anything when `current` is not a dotted version.
pub fn is_update_available(current: &str, latest: &str) -> bool {
    let current = current.trim();
    let latest = latest.trim();
    if !is_version_string(current) || !is_version_string(latest) {
        return false;
    }
    version_compare::compare_to(latest, current, Cmp::Gt).unwrap_or(false)
}
