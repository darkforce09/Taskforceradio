//! The answers the 1.0 plugin gives sync game calls, as constants and builders.
//!
//! **Role:** the reply strings of `processCommand` and `ts_info`
//! (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:72-162`, `:615-629`) and the speaking codes.
//!
//! **Position:** a module of `tfar_plugin_logic_1_0::game_commands`; used by the plugin's sync dispatch and by
//! [`crate::game_commands::Error::sync_reply`].
//!
//! **Signals and state:** none; constants and pure functions.
//!
//! **Invariants:** every string is byte for byte the original's; a speaking code is two ASCII
//! digits, talking first and receiving second; a bulk reply is one code and a tab per nickname,
//! the last tab kept.

#[cfg(test)]
#[path = "tests/replies.rs"]
mod tests;

/// The answer to a sync call of a name the plugin does not know (`CommandProcessor.cpp:80`).
pub const UNKNOWN_COMMAND: &str = "UNKNOWN COMMAND";

/// The answer to a sync call of an async-only command (`CommandProcessor.cpp:161`).
pub const ASYNC_COMMAND_SENT_IN_SYNC_CONTEXT: &str = "ASYNC COMMAND SENT IN SYNC CONTEXT";

/// The answer to `TS_INFO` with an unknown query (`CommandProcessor.cpp:628`).
pub const FAIL: &str = "FAIL";

/// The answer to `TS_INFO PING` (`CommandProcessor.cpp:625`).
pub const PONG: &str = "PONG";

/// The answer to `TS_INFO SERVER` and `TS_INFO SERVERUID` when TeamSpeak cannot give the value
/// (`reference/tfar_1_0/ts/src/Teamspeak.cpp:555`, `:568`).
pub const ERROR_GETTING_SERVER_NAME: &str = "ERROR_GETTING_SERVER_NAME";

/// The answer to `TS_INFO CHANNEL` when TeamSpeak cannot give the channel name: the empty text
/// (`reference/tfar_1_0/ts/src/Teamspeak.cpp:527-534`).
pub const CHANNEL_NAME_UNAVAILABLE: &str = "";

/// The speaking code of a client the plugin does not know (`CommandProcessor.cpp:92-95`).
pub const UNKNOWN_CLIENT_SPEAKING_CODE: [u8; 2] = *b"00";

/// The two-character speaking code of `POS` and `IS_SPEAKING` (`CommandProcessor.cpp:96-101`):
/// the first is `1` when the client talks (directly or on a radio), the second `1` when the local
/// player receives a transmission.
pub const fn speaking_code(talking: bool, receiving: bool) -> [u8; 2] {
    [digit(talking), digit(receiving)]
}

/// [`speaking_code`] as text: `00`, `01`, `10` or `11`.
pub const fn speaking_code_text(talking: bool, receiving: bool) -> &'static str {
    match (talking, receiving) {
        (false, false) => "00",
        (false, true) => "01",
        (true, false) => "10",
        (true, true) => "11",
    }
}

/// The answer to `IS_SPEAKING_BULK` (`CommandProcessor.cpp:103-144`): each code followed by a
/// tab, in nickname order, the last tab kept; the empty text for no nickname.
pub fn bulk_speaking_reply(codes: impl IntoIterator<Item = [u8; 2]>) -> String {
    let mut reply = String::new();
    for [talking, receiving] in codes {
        reply.push(char::from(talking));
        reply.push(char::from(receiving));
        reply.push('\t');
    }
    reply
}

/// `b'1'` for true, `b'0'` for false.
const fn digit(value: bool) -> u8 {
    if value { b'1' } else { b'0' }
}
