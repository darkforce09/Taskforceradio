//! The 1.0 `TRACK` custom variables: an SQF array `[[1,"name","value"],...]` read the way the
//! original's character-stream parser reads it.
//!
//! **Role:** turns the fourth `TRACK` token into the id, name and value triples that the `cvar`
//! parameter carries.
//!
//! **Position:** inside `piwik_tracking`; used by `track_request_1_0`. The specification is the
//! `parseCustomVariables` lambda in `reference/tfar_1_0/ts/src/task_force_radio.cpp`.
//!
//! **Signals and state:** none; a pure function of the text.
//!
//! **Invariants:** the reading matches the original character for character:
//! - whitespace is skipped everywhere, inside quoted names and values too (`std::istream >>
//!   char`), so `"Op Test"` reads as `OpTest`;
//! - the first `[` opens the outer array; every later `[` opens a variable;
//! - the id runs to the next `,` and parses like `std::stoi` (optional sign, leading digits);
//! - the name runs to the next `,` and the value to the next `]`, each with spaces and double
//!   quotes trimmed from both ends;
//! - an id `std::stoi` refuses voids the whole array.
//!
//! Text that ends inside a variable, where the original reads on forever, also voids the array.

#[cfg(test)]
#[path = "tests/custom_variables.rs"]
mod tests;

/// One Piwik custom variable from a `TRACK` message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomVariable {
    /// The slot number, printed as the original's `unsigned int`: `-1` reads as `4294967295`.
    pub id: u32,
    /// The variable name, trimmed of spaces and double quotes.
    pub name: String,
    /// The variable value, trimmed of spaces and double quotes.
    pub value: String,
}

/// Reads the custom-variable array; an empty list means the original would send nothing.
pub fn parse_custom_variables(text: &str) -> Vec<CustomVariable> {
    let mut stream = text
        .chars()
        .filter(|character| !is_c_whitespace(*character));
    read_custom_variables(&mut stream).unwrap_or_default()
}

/// The parser over the whitespace-free character stream; `None` voids the whole array.
fn read_custom_variables(stream: &mut impl Iterator<Item = char>) -> Option<Vec<CustomVariable>> {
    let mut variables = Vec::new();
    let Some(mut current) = stream.next() else {
        return Some(variables);
    };
    while current != '[' {
        match stream.next() {
            Some(next) => current = next,
            None => return Some(variables),
        }
    }
    let mut is_outer_bracket = true;
    loop {
        if current == '[' {
            if is_outer_bracket {
                is_outer_bracket = false;
                match stream.next() {
                    Some(next) => {
                        current = next;
                        continue;
                    }
                    None => break,
                }
            }
            variables.push(read_one_variable(stream)?);
        }
        match stream.next() {
            Some(next) => current = next,
            None => break,
        }
    }
    Some(variables)
}

/// Reads `1,"name","value"]` after a variable's `[`, consuming the closing `]`.
fn read_one_variable(stream: &mut impl Iterator<Item = char>) -> Option<CustomVariable> {
    let id_text = read_until(stream, ',')?;
    let id = parse_like_stoi(&id_text)?.cast_unsigned();
    let name = read_until(stream, ',')?;
    let value = read_until(stream, ']')?;
    Some(CustomVariable {
        id,
        name: trim_spaces_and_quotes(&name).to_owned(),
        value: trim_spaces_and_quotes(&value).to_owned(),
    })
}

/// Collects characters up to `end`, consuming it; `None` when the text ends first.
fn read_until(stream: &mut impl Iterator<Item = char>, end: char) -> Option<String> {
    let mut collected = String::new();
    loop {
        let next = stream.next()?;
        if next == end {
            return Some(collected);
        }
        collected.push(next);
    }
}

/// Parses like `std::stoi` in base 10: an optional sign, then at least one digit, stopping at the
/// first non-digit; `None` where `std::stoi` throws (no digit, or outside the `int` range).
fn parse_like_stoi(text: &str) -> Option<i32> {
    let (is_negative, unsigned) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let digit_count = unsigned.bytes().take_while(u8::is_ascii_digit).count();
    let digits = unsigned.get(..digit_count)?;
    if digits.is_empty() {
        return None;
    }
    let magnitude: i64 = digits.parse().ok()?;
    let value = if is_negative { -magnitude } else { magnitude };
    i32::try_from(value).ok()
}

/// Trims spaces and double quotes from both ends (the original's `trimWhitespaceColon`).
fn trim_spaces_and_quotes(text: &str) -> &str {
    text.trim_matches(|character| character == ' ' || character == '"')
}

/// The characters C's `isspace` accepts in the "C" locale, which `std::istream >> char` skips.
fn is_c_whitespace(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n' | '\u{0B}' | '\u{0C}' | '\r')
}
