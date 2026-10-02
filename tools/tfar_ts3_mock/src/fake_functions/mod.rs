//! The fake TeamSpeak functions and the table that holds them.

mod action_fakes;
mod connection_query_fakes;
pub(crate) mod fake_answers;
pub(crate) mod fake_table;
mod memory_and_log_fakes;
mod variable_fakes;

pub use fake_table::{FAKED_FUNCTIONS, FakedFunction, SLOT_COUNT};
pub use memory_and_log_fakes::mock_error_text;
