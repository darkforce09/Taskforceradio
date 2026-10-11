//! A code-only view of a Rust source file, for the gates that look for code patterns.
//!
//! **Role:** blanks comments and the contents of string and character literals, line by line, so
//! a pattern such as an inline test module or an unsafe block matches only real code and never a
//! comment or a string that happens to spell it.
//!
//! **Position:** used by `test-placement`, and `unsafe-allowlist`; pure.
//!
//! **Signals and state:** none; pure functions.
//!
//! **Invariants:** the view has exactly as many lines as the source and every kept character
//! stays in its column; literal delimiters stay, their contents become spaces.

#[cfg(test)]
#[path = "tests/rust_source.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LexState {
    Code,
    LineComment,
    BlockComment(usize),
    Text { escaped: bool },
    RawText { hashes: usize },
    Character { escaped: bool },
}

/// The source's lines with comments blanked and literal contents replaced by spaces.
pub(crate) fn code_lines(source: &str) -> Vec<String> {
    let chars: Vec<char> = source.chars().collect();
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut state = LexState::Code;
    let mut index = 0;
    while let Some(&character) = chars.get(index) {
        let next = chars.get(index + 1).copied();
        if character == '\n' {
            lines.push(std::mem::take(&mut current));
            if state == LexState::LineComment {
                state = LexState::Code;
            }
            index += 1;
            continue;
        }
        let (kept, new_state, consumed) = step(&chars, index, character, next, state);
        current.push_str(&kept);
        state = new_state;
        index += consumed;
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// One lexer step: the text kept for the consumed characters, the next state, and how many
/// characters were consumed.
fn step(
    chars: &[char],
    index: usize,
    character: char,
    next: Option<char>,
    state: LexState,
) -> (String, LexState, usize) {
    match state {
        LexState::Code => step_in_code(chars, index, character, next),
        LexState::LineComment => (" ".to_owned(), state, 1),
        LexState::BlockComment(depth) => match (character, next) {
            ('*', Some('/')) if depth == 1 => ("  ".to_owned(), LexState::Code, 2),
            ('*', Some('/')) => ("  ".to_owned(), LexState::BlockComment(depth - 1), 2),
            ('/', Some('*')) => ("  ".to_owned(), LexState::BlockComment(depth + 1), 2),
            _ => (" ".to_owned(), state, 1),
        },
        LexState::Text { escaped } => match character {
            _ if escaped => (" ".to_owned(), LexState::Text { escaped: false }, 1),
            '\\' => (" ".to_owned(), LexState::Text { escaped: true }, 1),
            '"' => ("\"".to_owned(), LexState::Code, 1),
            _ => (" ".to_owned(), state, 1),
        },
        LexState::RawText { hashes } => {
            if character == '"'
                && (1..=hashes).all(|offset| chars.get(index + offset) == Some(&'#'))
            {
                let mut kept = "\"".to_owned();
                kept.push_str(&"#".repeat(hashes));
                (kept, LexState::Code, 1 + hashes)
            } else {
                (" ".to_owned(), state, 1)
            }
        }
        LexState::Character { escaped } => match character {
            _ if escaped => (" ".to_owned(), LexState::Character { escaped: false }, 1),
            '\\' => (" ".to_owned(), LexState::Character { escaped: true }, 1),
            '\'' => ("'".to_owned(), LexState::Code, 1),
            _ => (" ".to_owned(), state, 1),
        },
    }
}

fn step_in_code(
    chars: &[char],
    index: usize,
    character: char,
    next: Option<char>,
) -> (String, LexState, usize) {
    let previous_is_identifier = index
        .checked_sub(1)
        .and_then(|before| chars.get(before))
        .is_some_and(|before| before.is_alphanumeric() || *before == '_');
    match (character, next) {
        ('/', Some('/')) => ("  ".to_owned(), LexState::LineComment, 2),
        ('/', Some('*')) => ("  ".to_owned(), LexState::BlockComment(1), 2),
        ('"', _) => ("\"".to_owned(), LexState::Text { escaped: false }, 1),
        ('r' | 'b', _) if !previous_is_identifier => {
            if let Some((prefix_length, hashes)) = raw_string_opening(chars, index) {
                let opening: String = chars
                    .get(index..index + prefix_length)
                    .map(|slice| slice.iter().collect())
                    .unwrap_or_default();
                (opening, LexState::RawText { hashes }, prefix_length)
            } else {
                (character.to_string(), LexState::Code, 1)
            }
        }
        ('\'', Some('\\')) => ("'".to_owned(), LexState::Character { escaped: false }, 1),
        ('\'', Some(_)) if chars.get(index + 2) == Some(&'\'') => {
            ("' '".to_owned(), LexState::Code, 3)
        }
        _ => (character.to_string(), LexState::Code, 1),
    }
}

/// When a raw string (`r"`, `r#"`, `br#"`) opens at `index`: the length of its opening and the
/// number of hashes.
fn raw_string_opening(chars: &[char], index: usize) -> Option<(usize, usize)> {
    let mut cursor = index;
    if chars.get(cursor) == Some(&'b') {
        cursor += 1;
    }
    if chars.get(cursor) != Some(&'r') {
        return None;
    }
    cursor += 1;
    let mut hashes = 0;
    while chars.get(cursor) == Some(&'#') {
        hashes += 1;
        cursor += 1;
    }
    (chars.get(cursor) == Some(&'"')).then_some((cursor + 1 - index, hashes))
}

/// Whether a raw source line is a `//` comment line, doc comments included.
pub(crate) fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with("//")
}
