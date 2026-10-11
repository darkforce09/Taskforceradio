//! The mod version a game link endpoint speaks for.

#[cfg(test)]
#[path = "tests/flavour.rs"]
mod tests;

/// The mod version of a game link endpoint, sent as the `HELLO` flavour byte.
///
/// The 0.9 and 1.0 pairs never talk to each other: a plugin refuses a `HELLO` of the other flavour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Flavour {
    /// The 0.9.12 mod: every call is sync.
    V0_9 = 9,
    /// The 1.0 mod: sync calls, `~` async calls and the focus state.
    V1_0 = 10,
}

impl Flavour {
    /// The byte this flavour is sent as in `HELLO`.
    pub const fn wire_value(self) -> u8 {
        self as u8
    }

    /// The flavour a `HELLO` flavour byte names, or `None` for a byte no flavour uses.
    pub const fn from_wire_value(value: u8) -> Option<Self> {
        match value {
            9 => Some(Self::V0_9),
            10 => Some(Self::V1_0),
            _ => None,
        }
    }
}

impl core::fmt::Display for Flavour {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::V0_9 => "0.9",
            Self::V1_0 => "1.0",
        })
    }
}
