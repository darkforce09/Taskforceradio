//! `NetId`: the Arma network id (`creator:object`) that names a radio tower.

use tfar_foundation::arma_values::parse_int;

#[cfg(test)]
#[path = "tests/net_id.rs"]
mod tests;

/// An Arma network object id, `creator:object` (`2:390`), as the radio tower messages carry it.
///
/// Two antennas are the same antenna when their ids are equal; an id whose object number is 0 is
/// the null id, which [`crate::antennas::AntennaManager`] ignores, as `AntennaManager::addAntenna` and
/// `removeAntenna` do (`reference/tfar_1_0/ts/src/antennaManager.cpp:49`, `:60`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NetId {
    creator: i32,
    object: i32,
}

impl NetId {
    /// The null id, `0:0`.
    pub const NULL: Self = Self {
        creator: 0,
        object: 0,
    };

    /// The id `creator:object`.
    pub const fn new(creator: i32, object: i32) -> Self {
        Self { creator, object }
    }

    /// The id in `text` (`2:390`). The colon is searched within `text` only, and each side is
    /// read as an Arma integer; a text without a colon, or with a side that is not a number, is
    /// the null id.
    pub fn parse(text: &str) -> Self {
        let Some((creator, object)) = text.split_once(':') else {
            return Self::NULL;
        };
        match (parse_int(creator), parse_int(object)) {
            (Ok(creator), Ok(object)) => Self { creator, object },
            _ => Self::NULL,
        }
    }

    /// The number before the colon: the machine that created the object.
    pub const fn creator(self) -> i32 {
        self.creator
    }

    /// The number after the colon: the object on that machine.
    pub const fn object(self) -> i32 {
        self.object
    }

    /// Whether this is the null id (object 0).
    pub const fn is_null(self) -> bool {
        self.object == 0
    }
}

impl core::fmt::Display for NetId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{}:{}", self.creator, self.object)
    }
}
