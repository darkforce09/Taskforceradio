//! Whole-name glob matching for README Contents entries: `*`, `?`, `[…]` and `{a,b}`.
//!
//! **Role:** decides whether a Contents entry such as `*.rs` or `{lib,main}.rs` names a child.
//!
//! **Position:** used by `readme-coverage`; pure.
//!
//! **Signals and state:** none; pure functions.
//!
//! **Invariants:** a pattern matches a whole name, never a part; a pattern without glob
//! characters matches only the identical name.

#[cfg(test)]
#[path = "tests/glob_pattern.rs"]
mod tests;

/// Whether the pattern holds any glob character.
pub(crate) fn is_glob(pattern: &str) -> bool {
    pattern.contains(['*', '?', '[', '{'])
}

/// Whether `pattern` matches the whole of `name`.
pub(crate) fn matches(pattern: &str, name: &str) -> bool {
    let name_chars: Vec<char> = name.chars().collect();
    expand_braces(pattern).iter().any(|alternative| {
        let pattern_chars: Vec<char> = alternative.chars().collect();
        match_from(&pattern_chars, &name_chars)
    })
}

/// Expands the first `{a,b}` group recursively into every alternative.
fn expand_braces(pattern: &str) -> Vec<String> {
    let Some(open) = pattern.find('{') else {
        return vec![pattern.to_owned()];
    };
    let Some(close_offset) = pattern.get(open..).and_then(|rest| rest.find('}')) else {
        return vec![pattern.to_owned()];
    };
    let close = open + close_offset;
    let before = pattern.get(..open).unwrap_or("");
    let inside = pattern.get(open + 1..close).unwrap_or("");
    let after = pattern.get(close + 1..).unwrap_or("");
    inside
        .split(',')
        .flat_map(|choice| expand_braces(&format!("{before}{choice}{after}")))
        .collect()
}

fn match_from(pattern: &[char], name: &[char]) -> bool {
    let Some((&first, pattern_rest)) = pattern.split_first() else {
        return name.is_empty();
    };
    match first {
        '*' => (0..=name.len()).any(|skip| {
            name.get(skip..)
                .is_some_and(|rest| match_from(pattern_rest, rest))
        }),
        '?' => name
            .split_first()
            .is_some_and(|(_, name_rest)| match_from(pattern_rest, name_rest)),
        '[' => match_class(pattern_rest, name),
        literal => name.split_first().is_some_and(|(&character, name_rest)| {
            character == literal && match_from(pattern_rest, name_rest)
        }),
    }
}

/// Matches a `[…]` class whose body starts at `pattern`, then the rest of the pattern.
fn match_class(pattern: &[char], name: &[char]) -> bool {
    let Some((&character, name_rest)) = name.split_first() else {
        return false;
    };
    let Some(close) = pattern.iter().position(|candidate| *candidate == ']') else {
        return false;
    };
    let body = pattern.get(..close).unwrap_or(&[]);
    let rest = pattern.get(close + 1..).unwrap_or(&[]);
    let (negated, body) = match body.split_first() {
        Some((&('!' | '^'), negated_body)) => (true, negated_body),
        _ => (false, body),
    };
    let mut is_member = false;
    let mut index = 0;
    while let Some(&low) = body.get(index) {
        if body.get(index + 1) == Some(&'-')
            && let Some(&high) = body.get(index + 2)
        {
            is_member |= low <= character && character <= high;
            index += 3;
            continue;
        }
        is_member |= low == character;
        index += 1;
    }
    is_member != negated && match_from(rest, name_rest)
}
