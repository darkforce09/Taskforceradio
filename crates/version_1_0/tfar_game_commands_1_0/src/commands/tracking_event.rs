//! `TrackingEvent`: the `TRACK` message, one usage-statistics event.
//!
//! **Role:** read the action, the player id and the custom variables of `TRACK`, the variables
//! read as the original's character loop reads them.
//!
//! **Position:** a module of the crate's `commands`; built on `MessageFields`; called by the
//! parser; the plugin hands the event to `tfar_web_requests`.
//!
//! **Signals and state:** none; pure parsing.
//!
//! **Invariants:** the message is exactly 4 tokens; the variables are read as
//! `parseCustomVariables` reads them (`reference/tfar_1_0/ts/src/task_force_radio.cpp:235-283`):
//! whitespace dropped everywhere (the loop reads with `operator>>`), each `[id,name,value]` group
//! split at its first two commas, name and value trimmed of quotes; a group that does not have
//! that shape, or an id that is not a number, rejects the message, where the original gives up
//! on the event.

use crate::error::Result;
use crate::fields::MessageFields;

#[cfg(test)]
#[path = "tests/tracking_event.rs"]
mod tests;

/// One custom variable of a tracking event: `[id, "name", value]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackerCustomVariable {
    /// The variable's slot number.
    pub id: u32,
    /// The variable's name, without quotes.
    pub name: String,
    /// The variable's value as text, without quotes.
    pub value: String,
}

/// One `TRACK` message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackingEvent {
    /// The action: `start`, `continue`, `beta` and the like.
    pub action: String,
    /// The player's id.
    pub uid: String,
    /// The custom variables, in message order. The original sends no request when there are
    /// none.
    pub custom_variables: Vec<TrackerCustomVariable>,
}

/// The `TRACK` message whose tokens are `fields`.
pub(crate) fn parse_tracking_event(fields: &MessageFields<'_>) -> Result<TrackingEvent> {
    fields.require_count(4)?;
    Ok(TrackingEvent {
        action: fields.text(1)?.to_owned(),
        uid: fields.text(2)?.to_owned(),
        custom_variables: fields.read(3, "custom variables", parse_custom_variables)?,
    })
}

/// The variables of `text`, an array of `[id,name,value]` groups, or `None` when a group is
/// malformed or there is no array at all.
fn parse_custom_variables(text: &str) -> Option<Vec<TrackerCustomVariable>> {
    let compact: String = text
        .chars()
        .filter(|character| !is_stream_whitespace(*character))
        .collect();
    let (_, mut rest) = compact.split_once('[')?;
    let mut variables = Vec::new();
    while let Some((_, group_start)) = rest.split_once('[') {
        let (group, after) = group_start.split_once(']')?;
        let (id, name_and_value) = group.split_once(',')?;
        let (name, value) = name_and_value.split_once(',')?;
        variables.push(TrackerCustomVariable {
            id: id.parse().ok()?,
            name: trim_quotes(name).to_owned(),
            value: trim_quotes(value).to_owned(),
        });
        rest = after;
    }
    Some(variables)
}

/// Whether `operator>>` skips `character` as whitespace in the C locale.
const fn is_stream_whitespace(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n' | '\x0B' | '\x0C' | '\r')
}

/// `text` without the quotes at either end (`trimWhitespaceColon`).
fn trim_quotes(text: &str) -> &str {
    text.trim_matches(|character| character == '"' || character == ' ')
}
