//! The command names of the 0.9.12 game messages: the first tab-separated field, as
//! `processGameCommand` in `reference/tfar_0_9_12/ts/src/plugin.cpp` compares it.

/// A query about TeamSpeak: `TS_INFO\t<SERVER|CHANNEL|PING>`.
pub const TS_INFO: &str = "TS_INFO";
/// A player died: `KILLED\t<nickname>`.
pub const KILLED: &str = "KILLED";
/// The radios on loudspeaker near the player: `SPEAKERS\t<records>`.
pub const SPEAKERS: &str = "SPEAKERS";
/// The player's radios were taken: `RELEASE_ALL_TANGENTS\t<nickname>`.
pub const RELEASE_ALL_TANGENTS: &str = "RELEASE_ALL_TANGENTS";
/// The mission handshake: `VERSION\t<addon>\t<serious channel>\t<password>`.
pub const VERSION: &str = "VERSION";
/// An analytics request: `TRACK\t<path>`.
pub const TRACK: &str = "TRACK";
/// One player's position and abilities: `POS` and 13 fields.
pub const POS: &str = "POS";
/// A short-range push-to-talk change: `TANGENT` and 4 fields.
pub const TANGENT: &str = "TANGENT";
/// A long-range push-to-talk change: `TANGENT_LR` and 4 fields.
pub const TANGENT_LR: &str = "TANGENT_LR";
/// A diver radio push-to-talk change: `TANGENT_DD` and 4 fields.
pub const TANGENT_DD: &str = "TANGENT_DD";
/// The player's radios and settings: `FREQ` and 13 fields.
pub const FREQ: &str = "FREQ";
/// Whether a player is talking: `IS_SPEAKING\t<nickname>`.
pub const IS_SPEAKING: &str = "IS_SPEAKING";
