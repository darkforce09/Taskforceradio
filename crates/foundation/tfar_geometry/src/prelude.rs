//! The names a caller of `tfar_geometry` imports with `use tfar_geometry::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::angle::Angle;
pub use crate::crate_role::CRATE_ROLE;
pub use crate::direction::Direction;
pub use crate::position::Position;
pub use crate::rotation::rotate_about_z;
pub use crate::underwater_distance::distance_underwater;
