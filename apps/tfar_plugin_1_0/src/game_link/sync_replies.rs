//! The answers to the game's sync calls: `TS_INFO`, `POS`, `IS_SPEAKING` and
//! `IS_SPEAKING_BULK`, and the fixed answers of everything else.
//!
//! **Role:** `CommandProcessor::processCommand` (`reference/tfar_1_0/ts/src/
//! CommandProcessor.cpp:72-163`, `:600-629`), with the reply strings of
//! `tfar_game_commands_1_0::replies`.
//!
//! **Position:** called by `LinkHandler::sync_request` on the link's session thread.
//!
//! **Signals and state:** reads the current server's session under its lock; the `POS` payload is
//! applied right after the reply is taken, on the same thread.
//!
//! **Invariants:** a malformed message gets its command's error reply (`00` for `POS`), never a
//! panic; TeamSpeak is asked only with the server lock released.

use tfar_game_commands_1_0::prelude::{GameCommand, TsInfoQuery, parse};
use tfar_game_commands_1_0::replies::{
    ASYNC_COMMAND_SENT_IN_SYNC_CONTEXT, CHANNEL_NAME_UNAVAILABLE, ERROR_GETTING_SERVER_NAME,
    UNKNOWN_CLIENT_SPEAKING_CODE, bulk_speaking_reply,
};

use crate::game_link::async_commands::apply_position;
use crate::plugin_lifecycle::PLUGIN_VERSION;
use crate::runtime::PluginRuntime;

/// The two-character code as text.
fn code_text(code: [u8; 2]) -> String {
    code.iter().copied().map(char::from).collect()
}

/// The speaking code of `nickname` on the current server; `00` without one.
fn speaking_code(runtime: &PluginRuntime, nickname: &str) -> [u8; 2] {
    runtime
        .current_server()
        .map_or(UNKNOWN_CLIENT_SPEAKING_CODE, |(_, state)| {
            state.lock().session.speaking_code(nickname)
        })
}

/// The answer to the sync call `text`.
pub fn sync_reply(runtime: &PluginRuntime, text: &str) -> String {
    let command = match parse(text) {
        Ok(command) => command,
        Err(error) => {
            tracing::warn!(%error, "a sync game message was rejected");
            return error.sync_reply().to_owned();
        }
    };
    if let Some(fixed) = command.fixed_sync_reply() {
        return fixed.to_owned();
    }
    match command {
        GameCommand::TsInfo(query) => ts_info_reply(runtime, &query),
        GameCommand::Position(update) => {
            let code = speaking_code(runtime, &update.nickname);
            apply_position(runtime, &update);
            code_text(code)
        }
        GameCommand::IsSpeaking { nickname } => code_text(speaking_code(runtime, &nickname)),
        GameCommand::IsSpeakingBulk(nicknames) => {
            let codes: Vec<[u8; 2]> = match runtime.current_server() {
                Some((_, state)) => {
                    let state = state.lock();
                    nicknames
                        .iter()
                        .map(|nickname| state.session.speaking_code(nickname))
                        .collect()
                }
                None => vec![UNKNOWN_CLIENT_SPEAKING_CODE; nicknames.len()],
            };
            bulk_speaking_reply(codes)
        }
        _ => ASYNC_COMMAND_SENT_IN_SYNC_CONTEXT.to_owned(),
    }
}

/// The answer to `TS_INFO` with `query`, read from TeamSpeak.
fn ts_info_reply(runtime: &PluginRuntime, query: &TsInfoQuery) -> String {
    let ts = runtime.ts();
    let server = ts.current_server();
    let own_channel = || {
        ts.own_client_id(server)
            .and_then(|own| ts.channel_of_client(server, own))
    };
    match query {
        TsInfoQuery::Server => ts
            .server_name(server)
            .unwrap_or_else(|| ERROR_GETTING_SERVER_NAME.to_owned()),
        TsInfoQuery::ServerUid => ts
            .server_unique_identifier(server)
            .unwrap_or_else(|| ERROR_GETTING_SERVER_NAME.to_owned()),
        TsInfoQuery::Channel => own_channel()
            .and_then(|channel| ts.channel_name(server, channel))
            .unwrap_or_else(|| CHANNEL_NAME_UNAVAILABLE.to_owned()),
        TsInfoQuery::ChannelId => own_channel().map_or(0, |channel| channel.0).to_string(),
        TsInfoQuery::Version => PLUGIN_VERSION.to_owned(),
        TsInfoQuery::Ping | TsInfoQuery::Other(_) => query.fixed_reply().unwrap_or("").to_owned(),
    }
}
