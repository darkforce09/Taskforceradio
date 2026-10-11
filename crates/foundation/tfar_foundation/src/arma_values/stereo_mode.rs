//! `StereoMode`: which ear a radio plays into.

#[cfg(test)]
#[path = "tests/stereo_mode.rs"]
mod tests;

/// Which ear a radio plays into, as the frequency lists number it (`stereoMode` in
/// `reference/tfar_1_0/ts/src/common.hpp`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum StereoMode {
    /// Both ears: wire value 0, and the reading of any unknown value.
    #[default]
    Both,
    /// The left ear only: wire value 1.
    Left,
    /// The right ear only: wire value 2.
    Right,
}

impl StereoMode {
    /// The mode for a rounded wire value: 1 is left, 2 is right, and 0 or any other value is both.
    pub const fn from_wire_value(value: i32) -> Self {
        match value {
            1 => Self::Left,
            2 => Self::Right,
            _ => Self::Both,
        }
    }

    /// The wire value of the mode: 0, 1 or 2.
    pub const fn wire_value(self) -> i32 {
        match self {
            Self::Both => 0,
            Self::Left => 1,
            Self::Right => 2,
        }
    }
}
