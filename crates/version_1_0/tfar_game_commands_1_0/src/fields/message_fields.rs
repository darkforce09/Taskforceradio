//! `MessageFields`: the tokens of one game message, read by position with checked counts.
//!
//! **Role:** split a message the way the 1.0 plugin splits it and hand each command parser its
//! fields by index, turning a short message or a malformed field into a typed error that names
//! the command and the field.
//!
//! **Position:** a module of the crate's `fields`; built on `tfar_arma_values`; used by every
//! command parser.
//!
//! **Signals and state:** none; a borrowed view of one message.
//!
//! **Invariants:** index 0 is the command name, so a field's index is the original's
//! `tokens[i]`; every read goes through `get`, so a missing token is an error, never a panic; the
//! split keeps every field, a trailing empty one included, as `helpers::split` does
//! (`reference/tfar_1_0/ts/src/helpers.cpp:245-262`).

use tfar_arma_values::{parse_int, parse_number, sanitize_nickname};

use crate::command_name::CommandName;
use crate::error::{Error, Result};
use crate::fields::arma_truth::is_true;

#[cfg(test)]
#[path = "tests/message_fields.rs"]
mod tests;

/// `text` split at every `separator`, every field kept: an empty text is one empty field and a
/// trailing separator gives a trailing empty field, as `helpers::split` in
/// `reference/tfar_1_0/ts/src/helpers.cpp:245-262` splits.
pub(crate) fn split_keeping_every_field(text: &str, separator: char) -> Vec<&str> {
    text.split(separator).collect()
}

/// The tokens of one game message and the command they belong to.
#[derive(Debug)]
pub(crate) struct MessageFields<'a> {
    command: CommandName,
    tokens: Vec<&'a str>,
}

impl<'a> MessageFields<'a> {
    /// The fields of `tokens`, the message of `command`; `tokens[0]` is the command name.
    pub(crate) const fn new(command: CommandName, tokens: Vec<&'a str>) -> Self {
        Self { command, tokens }
    }

    /// The number of tokens, the command name included.
    pub(crate) fn count(&self) -> usize {
        self.tokens.len()
    }

    /// The tokens after the command name.
    pub(crate) fn after_name(&self) -> &[&'a str] {
        self.tokens.get(1..).unwrap_or_default()
    }

    /// Checks that the message has exactly `expected` tokens, the command name included.
    pub(crate) fn require_count(&self, expected: usize) -> Result<()> {
        self.require(self.count() == expected, expected, expected)
    }

    /// Checks that the message has at least `minimum` tokens, the command name included.
    pub(crate) fn require_at_least(&self, minimum: usize) -> Result<()> {
        self.require(self.count() >= minimum, minimum, usize::MAX)
    }

    /// Checks that the message has between `minimum` and `maximum` tokens.
    pub(crate) fn require_between(&self, minimum: usize, maximum: usize) -> Result<()> {
        self.require(
            (minimum..=maximum).contains(&self.count()),
            minimum,
            maximum,
        )
    }

    fn require(&self, holds: bool, minimum: usize, maximum: usize) -> Result<()> {
        if holds {
            Ok(())
        } else {
            Err(Error::WrongTokenCount {
                command: self.command,
                minimum,
                maximum,
                found: self.count(),
            })
        }
    }

    /// Token `index` as sent.
    pub(crate) fn text(&self, index: usize) -> Result<&'a str> {
        self.tokens
            .get(index)
            .copied()
            .ok_or(Error::WrongTokenCount {
                command: self.command,
                minimum: index.saturating_add(1),
                maximum: usize::MAX,
                found: self.count(),
            })
    }

    /// Token `index` as a nickname in the form the plugin compares
    /// ([`tfar_arma_values::sanitize_nickname`], `convertNickname`).
    pub(crate) fn nickname(&self, index: usize) -> Result<String> {
        self.text(index).map(sanitize_nickname)
    }

    /// Token `index` read as `helpers::isTrue` reads it.
    pub(crate) fn truth(&self, index: usize) -> Result<bool> {
        self.text(index).map(is_true)
    }

    /// Token `index` as a finite number; `field` names it in the error.
    pub(crate) fn number(&self, index: usize, field: &'static str) -> Result<f64> {
        let text = self.text(index)?;
        parse_number(text).map_err(|_| self.malformed(field, text))
    }

    /// Token `index` as a number rounded to the nearest integer (`parseArmaNumberToInt`).
    pub(crate) fn integer(&self, index: usize, field: &'static str) -> Result<i32> {
        let text = self.text(index)?;
        parse_int(text).map_err(|_| self.malformed(field, text))
    }

    /// Token `index` read by `reader`, which gives `None` for a malformed text.
    pub(crate) fn read<T>(
        &self,
        index: usize,
        field: &'static str,
        reader: impl FnOnce(&'a str) -> Option<T>,
    ) -> Result<T> {
        let text = self.text(index)?;
        reader(text).ok_or_else(|| self.malformed(field, text))
    }

    /// The error for a malformed `field` whose text is `value`.
    pub(crate) fn malformed(&self, field: &'static str, value: &str) -> Error {
        Error::Malformed {
            command: self.command,
            field,
            value: value.to_owned(),
        }
    }
}
