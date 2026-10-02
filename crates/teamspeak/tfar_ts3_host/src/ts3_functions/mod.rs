//! [`Ts3`], the safe handle over TeamSpeak's function table, split by SDK function group.

mod client_actions;
mod client_variables;
mod connection_queries;
mod handle;
pub(crate) mod owned_memory;
mod sound_and_paths;

pub use handle::Ts3;

#[cfg(test)]
#[path = "tests/ts3_calls.rs"]
mod ts3_calls_tests;

#[cfg(test)]
#[path = "tests/owned_memory.rs"]
mod owned_memory_tests;
