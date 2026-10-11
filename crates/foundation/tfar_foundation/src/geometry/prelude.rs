//! The names a caller of `tfar_foundation::geometry` imports with `use crate::geometry::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::geometry::angle::Angle;
pub use crate::geometry::direction::Direction;
pub use crate::geometry::position::Position;
pub use crate::geometry::rotation::rotate_about_z;
pub use crate::geometry::underwater_distance::distance_underwater;
