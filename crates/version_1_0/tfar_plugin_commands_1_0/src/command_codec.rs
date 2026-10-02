//! `encode` and `decode`: plugin commands to and from the TAB-separated text TeamSpeak carries.
//!
//! **Role:** write each `PluginCommand` byte for byte as the Windows 1.0 plugin writes it, and
//! read the text a Windows or Linux 1.0 plugin sent back into its command.
//!
//! **Position:** a module of `tfar_plugin_commands_1_0`; built on `PluginCommand` and
//! `tfar_arma_values`; called by `tfar_plugin_1_0` before `sendPluginCommand` and in
//! `onPluginCommandEvent`.
//!
//! **Signals and state:** none; pure functions, callable from any thread.
//!
//! **Invariants:** the split keeps every field (`helpers::split`); a command is recognised by
//! its name and token count together, as `processPluginCommand` does
//! (`reference/tfar_1_0/ts/src/plugin.cpp:1026-1068`); a tangent's fields are read by position
//! relative to its token count, so the 8-token form (no class name) reads its radio flags from
//! tokens 5 and 6; decoding then encoding a decoded command gives the same bytes.

use tfar_arma_values::RadioSubtype;
use tfar_arma_values::parse_int;

use crate::error::{Error, Result};
use crate::plugin_command::{PluginCommand, TangentBroadcast, TangentKind, TeamspeakClientId};

#[cfg(test)]
#[path = "tests/command_codec.rs"]
mod tests;

/// The name of the release-all message.
const RELEASE_ALL_TANGENTS: &str = "RELEASE_ALL_TANGENTS";

/// The name of the volume message.
const VOLUME: &str = "VOLUME";

/// The name of the volume request.
const REQUEST_VOLUME: &str = "REQVOL";

/// The state text of a press; any other text is a release.
const PRESSED: &str = "PRESSED";

/// The state text a release is encoded with.
const RELEASED: &str = "RELEASED";

/// `command` as the text the original sends with `sendPluginCommand`:
///
/// - a tangent: `kind\tPRESSED|RELEASED\tfrequency\trange\tsubtype[\tclassname]\tsw\tdd\tnick`,
///   the flags `1` or `0` (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:326`);
/// - `RELEASE_ALL_TANGENTS\tnick` (`CommandProcessor.cpp:378`);
/// - `VOLUME\tnick\tvolume\ttrue|false`, the volume in decimal (`plugin.cpp:841`, `:1062`);
/// - `REQVOL\tclientId`.
pub fn encode(command: &PluginCommand) -> String {
    match command {
        PluginCommand::Tangent(tangent) => encode_tangent(tangent),
        PluginCommand::ReleaseAllTangents { nickname } => {
            format!("{RELEASE_ALL_TANGENTS}\t{nickname}")
        }
        PluginCommand::Volume {
            nickname,
            voice_volume,
            is_talking,
        } => format!("{VOLUME}\t{nickname}\t{voice_volume}\t{is_talking}"),
        PluginCommand::RequestVolume { client_id } => {
            format!("{REQUEST_VOLUME}\t{}", client_id.0)
        }
    }
}

/// The text of a tangent broadcast.
fn encode_tangent(tangent: &TangentBroadcast) -> String {
    let state = if tangent.is_pressed {
        PRESSED
    } else {
        RELEASED
    };
    let mut text = format!(
        "{}\t{state}\t{}\t{}\t{}",
        tangent.kind.wire_name(),
        tangent.frequency,
        tangent.range_text,
        tangent.subtype.as_str(),
    );
    if let Some(classname) = &tangent.radio_classname {
        text.push('\t');
        text.push_str(classname);
    }
    text.push('\t');
    text.push_str(flag_text(tangent.can_use_sw_radio));
    text.push('\t');
    text.push_str(flag_text(tangent.can_use_dd_radio));
    text.push('\t');
    text.push_str(&tangent.nickname);
    text
}

/// `1` or `0`, as the tangent flags are written.
const fn flag_text(flag: bool) -> &'static str {
    if flag { "1" } else { "0" }
}

/// The command in `text`, a plugin command as received.
///
/// # Errors
///
/// [`Error::Unrecognised`] when no command has that name and token count (a tangent of 8 or 9
/// tokens, `RELEASE_ALL_TANGENTS` and `REQVOL` of 2, `VOLUME` of 4), and [`Error::Malformed`]
/// when a range, volume or client id is not a number.
pub fn decode(text: &str) -> Result<PluginCommand> {
    let tokens: Vec<&str> = text.split('\t').collect();
    let unrecognised = || Error::Unrecognised {
        text: text.to_owned(),
    };
    match tokens.as_slice() {
        [name, state, frequency, range, subtype, rest @ ..]
            if rest.len() == 3 || rest.len() == 4 =>
        {
            let kind = TangentKind::from_wire_name(name).ok_or_else(unrecognised)?;
            let (radio_classname, sw, dd, nickname) = match rest {
                [sw, dd, nickname] => (None, sw, dd, nickname),
                [classname, sw, dd, nickname] => (Some(*classname), sw, dd, nickname),
                _ => return Err(unrecognised()),
            };
            Ok(PluginCommand::Tangent(TangentBroadcast {
                kind,
                is_pressed: *state == PRESSED,
                frequency: (*frequency).to_owned(),
                range: parse_int(range).map_err(|_| malformed(kind.wire_name(), "range", range))?,
                range_text: (*range).to_owned(),
                subtype: RadioSubtype::parse(subtype),
                radio_classname: radio_classname.map(str::to_owned),
                can_use_sw_radio: *sw == "1",
                can_use_dd_radio: *dd == "1",
                nickname: (*nickname).to_owned(),
            }))
        }
        [name, nickname] if *name == RELEASE_ALL_TANGENTS => {
            Ok(PluginCommand::ReleaseAllTangents {
                nickname: (*nickname).to_owned(),
            })
        }
        [name, nickname, volume, talking] if *name == VOLUME => Ok(PluginCommand::Volume {
            nickname: (*nickname).to_owned(),
            voice_volume: volume
                .parse()
                .map_err(|_| malformed(VOLUME, "voice volume", volume))?,
            is_talking: *talking == "true" || *talking == "1",
        }),
        [name, client_id] if *name == REQUEST_VOLUME => Ok(PluginCommand::RequestVolume {
            client_id: TeamspeakClientId(
                client_id
                    .parse()
                    .map_err(|_| malformed(REQUEST_VOLUME, "client id", client_id))?,
            ),
        }),
        _ => Err(unrecognised()),
    }
}

/// The error for a malformed `field` of `command` whose text is `value`.
fn malformed(command: &'static str, field: &'static str, value: &str) -> Error {
    Error::Malformed {
        command,
        field,
        value: value.to_owned(),
    }
}
