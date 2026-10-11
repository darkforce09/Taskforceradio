//! The foundation every other crate builds on: [`geometry`] (positions, directions and distances in
//! Arma's frame) and [`arma_values`] (the parsers for the strings Arma sends). Pure functions and
//! value types, callable from any thread.

pub mod arma_values;
pub mod geometry;
