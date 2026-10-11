//! `NetworkObjectId`: an Arma `netId` such as `2:390`, as the radio tower messages carry it.

#[cfg(test)]
#[path = "tests/network_object_id.rs"]
mod tests;

/// An Arma network object id, `creator:object`, read as `NetID` in
/// `reference/tfar_1_0/ts/src/datatypes.hpp:187-212` reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct NetworkObjectId {
    /// The number before the colon: the machine that created the object.
    pub creator: i32,
    /// The number after the colon: the object on that machine; 0 for a null id.
    pub object: i32,
}

impl NetworkObjectId {
    /// The id in `text`. Each number is read as `atoi` reads it (leading spaces, an optional sign,
    /// the digits that follow; 0 when there are none), and a text without a colon is the null id
    /// `0:0`. The colon is searched within `text` only.
    pub fn parse(text: &str) -> Self {
        match text.split_once(':') {
            Some((creator, object)) => Self {
                creator: leading_integer(creator),
                object: leading_integer(object),
            },
            None => Self::default(),
        }
    }

    /// Whether this is the null id (object 0), which the antenna manager ignores
    /// (`reference/tfar_1_0/ts/src/antennaManager.cpp:49`, `:60`).
    pub const fn is_null(self) -> bool {
        self.object == 0
    }
}

/// The integer at the start of `text`, as `atoi` reads it; saturates instead of overflowing.
fn leading_integer(text: &str) -> i32 {
    let trimmed = text.trim_start_matches([' ', '\t', '\n', '\x0B', '\x0C', '\r']);
    let (negative, digits) = match trimmed.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    let mut value: i32 = 0;
    for digit in digits.chars().map_while(|character| character.to_digit(10)) {
        let digit = i32::try_from(digit).unwrap_or(0);
        value = value.saturating_mul(10);
        value = if negative {
            value.saturating_sub(digit)
        } else {
            value.saturating_add(digit)
        };
    }
    value
}
