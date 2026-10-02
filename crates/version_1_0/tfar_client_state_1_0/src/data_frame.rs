//! `DataFrame`: the game's frame counter, which tells a live client from a stale one.

#[cfg(test)]
#[path = "tests/data_frame.rs"]
mod tests;

/// A value of the game's data-frame counter (`DFRAME`), or the invalid frame.
///
/// The original keeps an `int` with `INT_MAX` as the invalid value
/// (`reference/tfar_1_0/ts/src/common.hpp:39`); here the invalid frame is its own state, so no
/// arithmetic ever overflows it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DataFrame(Option<u32>);

impl DataFrame {
    /// The invalid frame: no game, a killed client, or an expired one.
    pub const INVALID: Self = Self(None);

    /// The frame a game starts at (`onGameConnected`).
    pub const FIRST: Self = Self(Some(0));

    /// The frame numbered `number`.
    pub const fn new(number: u32) -> Self {
        Self(Some(number))
    }

    /// The frame number, or `None` for the invalid frame.
    pub const fn number(self) -> Option<u32> {
        self.0
    }

    /// Whether this is a real frame.
    pub const fn is_valid(self) -> bool {
        self.0.is_some()
    }

    /// The frame after this one; the invalid frame stays invalid, and the last frame stays the
    /// last.
    #[must_use]
    pub const fn next(self) -> Self {
        match self.0 {
            Some(number) => Self(Some(number.saturating_add(1))),
            None => Self::INVALID,
        }
    }

    /// Whether both frames are valid and at most one apart, the liveness rule of
    /// `clientData::isAlive` (`reference/tfar_1_0/ts/src/clientData.cpp:136-142`).
    pub const fn is_within_one_of(self, other: Self) -> bool {
        match (self.0, other.0) {
            (Some(mine), Some(theirs)) => mine.abs_diff(theirs) <= 1,
            _ => false,
        }
    }
}
