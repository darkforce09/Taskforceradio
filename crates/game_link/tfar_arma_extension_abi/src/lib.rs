//! The `RVExtension` exports and the Win32 calls an Arma extension DLL needs.
//!
//! **Role:** the Arma side of an extension DLL: the [`arma_extension!`] macro that writes the
//! `RVExtension` and `RVExtensionVersion` exports, [`write_reply`] that truncates a reply to Arma's
//! buffer, and the two Win32 queries an extension makes: [`arma_command_line_contains`] (`-tfdebug`)
//! and [`foreground_window_is_arma`] (the `FOCUS` frame).
//!
//! **Position:** `crates/game_link`, tier 0, depending on no workspace crate (`windows-sys` on
//! Windows); used by `tfar_extension_0_9`, `tfar_extension_1_0`. Only `x86_64` Windows is a supported
//! DLL target: there `extern "system"` is the C ABI, so the exports are undecorated. No 32-bit build
//! is supported.
//!
//! **Signals and state:** none; every call runs on Arma's calling thread and keeps nothing.
//!
//! **Invariants:** a reply never writes past the buffer Arma passes and always ends in a NUL when the
//! buffer has room for one; a host pointer is checked for null before it is read and is not kept
//! after the call.

// This crate is the Arma C ABI and the Win32 calls; each `unsafe` block carries a `// SAFETY:` comment.
#![allow(
    unsafe_code,
    reason = "the Arma C ABI and the Win32 calls live in this crate; each `unsafe` block carries a `// SAFETY:` comment"
)]

mod command_line;
mod error;
mod extension_exports;
mod foreground_window;
pub mod prelude;
mod reply_buffer;

pub use command_line::arma_command_line_contains;
pub use error::{Error, Result};
pub use extension_exports::{answer_extension_call, answer_extension_version};
pub use foreground_window::foreground_window_is_arma;
pub use reply_buffer::write_reply;
