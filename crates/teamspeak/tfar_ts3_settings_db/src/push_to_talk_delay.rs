//! [`PushToTalkDelay`]: the push-to-talk release delay of TeamSpeak's default capture profile, and
//! the parser of the profile text that holds it.
//!
//! **Role:** turns the `Capture/Default/PreProcessing` value (one `key=value` per line) into the
//! two settings the plugins use: whether the delay is on and how long it is.
//!
//! **Position:** called by [`read_push_to_talk_delay`](crate::read_push_to_talk_delay) on the text
//! it reads; pure, so the plugins' tests can call it on literal text.
//!
//! **Signals and state:** none; pure functions.
//!
//! **Invariants:** the line rules of `reference/tfar_1_0/ts/src/plugin.cpp:259` and
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:2191`: `delay_ptt=true` turns the delay on and
//! `delay_ptt_msecs=<n>` sets its length, the last line of each kind winning; lines end in `\n` or
//! `\r\n`; a length that is not a whole number of milliseconds is an error, never a crash.

use std::time::Duration;

use crate::{Error, Result};

/// The line that turns the delay on.
const DELAY_ENABLED_LINE: &str = "delay_ptt=true";
/// The key of the line that holds the delay in milliseconds.
const DELAY_MILLISECONDS_KEY: &str = "delay_ptt_msecs";

#[cfg(test)]
#[path = "tests/push_to_talk_delay.rs"]
mod tests;

/// How long TeamSpeak keeps transmitting after the push-to-talk key is released.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PushToTalkDelay {
    /// Whether the profile turns the delay on (`delay_ptt=true`).
    pub enabled: bool,
    /// The stored delay (`delay_ptt_msecs`), kept even when the delay is off.
    pub milliseconds: u32,
}

impl PushToTalkDelay {
    /// The delay the plugins apply: the stored length when enabled, zero otherwise.
    pub fn effective(self) -> Duration {
        if self.enabled {
            Duration::from_millis(u64::from(self.milliseconds))
        } else {
            Duration::ZERO
        }
    }

    /// Parses the text of the `Capture/Default/PreProcessing` profile.
    ///
    /// Lines other than the two the plugins read are ignored, so a profile without them is a
    /// disabled zero delay.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidMilliseconds`] when a `delay_ptt_msecs` line holds no `=` or no whole
    /// number of milliseconds.
    pub fn parse_pre_processing(profile: &str) -> Result<Self> {
        let mut delay = Self::default();
        for line in profile.lines() {
            let line = line.trim();
            if line == DELAY_ENABLED_LINE {
                delay.enabled = true;
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                if line == DELAY_MILLISECONDS_KEY {
                    return Err(invalid_milliseconds(line));
                }
                continue;
            };
            if key.trim() == DELAY_MILLISECONDS_KEY {
                delay.milliseconds = value
                    .trim()
                    .parse::<u32>()
                    .map_err(|_| invalid_milliseconds(line))?;
            }
        }
        Ok(delay)
    }
}

/// The error for the milliseconds `line`.
fn invalid_milliseconds(line: &str) -> Error {
    Error::InvalidMilliseconds {
        line: line.to_owned(),
    }
}
