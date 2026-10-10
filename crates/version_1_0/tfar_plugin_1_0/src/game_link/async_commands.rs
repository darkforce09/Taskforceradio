//! The async game messages applied in arrival order: positions, radios, kills, data frames,
//! speakers, tangents, settings, radio towers, tracking and the diagnostics dump.
//!
//! **Role:** `CommandProcessor::processAsynchronousCommand` (`reference/tfar_1_0/ts/src/
//! CommandProcessor.cpp:224-457`): each parsed message goes into the current server's state
//! through the apply functions of `tfar_client_state_1_0`, and what they return is carried out in
//! TeamSpeak after the lock is released.
//!
//! **Position:** called by `LinkHandler::async_message` on the link's async worker, which also
//! runs the sync `POS` calls the sync path queues there.
//!
//! **Signals and state:** the current server's state under its lock; the settings snapshot.
//!
//! **Invariants:** a message for a server with no state is dropped, as the original returns
//! without a client directory; the DSP objects a change needs are built here, off the audio
//! thread.

use std::time::Instant;

use tfar_antennas_1_0::prelude::{Antenna, NetId};
use tfar_game_commands_1_0::prelude::{FrequencyReport, GameCommand, PositionUpdate, TowerRecord};
use tfar_web_requests::prelude::{build_track_request_1_0, spawn_cancellable_track};

use crate::diagnostics::debug_dump::write_debug_dump;
use crate::game_link::local_tangent::{apply_local_tangent, release_all_tangents};
use crate::game_link::settings_apply::apply_set_config;
use crate::runtime::PluginRuntime;
use crate::runtime::mute_updates::{decide_client_mute, decide_killed_mutes};
use crate::runtime::server_state::host_client_id;

/// The shortest game name TeamSpeak accepts as a nickname.
pub const MINIMUM_NICKNAME_LENGTH: usize = 3;

/// The field of a `TRACK` message that holds the custom variables, as sent.
const TRACK_CUSTOM_VARIABLES_FIELD: usize = 3;

/// Applies the parsed async message `command`; `text` is the message as sent.
pub fn apply_async_message(runtime: &PluginRuntime, text: &str, command: GameCommand) {
    match command {
        GameCommand::Position(update) => apply_position(runtime, &update),
        GameCommand::Frequencies(report) => apply_frequencies(runtime, &report),
        GameCommand::Killed { nickname } => apply_killed(runtime, &nickname),
        GameCommand::DataFrame => {
            if let Some((_, state)) = runtime.current_server() {
                state.lock().session.next_data_frame();
            }
        }
        GameCommand::Speakers(records) => {
            if let Some((_, state)) = runtime.current_server() {
                let settings = runtime.settings();
                let mut state = state.lock();
                state.session.apply_speakers(&records);
                runtime.ensure_every_sender_dsp(&mut state, &settings);
            }
        }
        GameCommand::Tangent(press) => apply_local_tangent(runtime, &press),
        GameCommand::ReleaseAllTangents { nickname } => release_all_tangents(runtime, &nickname),
        GameCommand::SetConfig(assignment) => apply_set_config(runtime, &assignment),
        GameCommand::RadioTowerAdd(records) => add_towers(runtime, &records),
        GameCommand::RadioTowerRemove(ids) => {
            if let Some((_, state)) = runtime.current_server() {
                let ids = ids.iter().map(|id| NetId::new(id.creator, id.object));
                state.lock().antennas.remove(ids);
            }
        }
        GameCommand::Track(event) => {
            let custom_variables = text
                .split('\t')
                .nth(TRACK_CUSTOM_VARIABLES_FIELD)
                .unwrap_or_default();
            match build_track_request_1_0(&event.action, &event.uid, custom_variables) {
                Ok(url) => match spawn_cancellable_track(url) {
                    Ok(request) => runtime.web_requests.keep(request),
                    Err(error) => tracing::warn!(%error, "the tracking request did not start"),
                },
                Err(error) => tracing::debug!(%error, "no tracking request for this event"),
            }
        }
        GameCommand::CollectDebugInfo => write_debug_dump(runtime),
        GameCommand::MissionEnd
        | GameCommand::TsInfo(_)
        | GameCommand::IsSpeaking { .. }
        | GameCommand::IsSpeakingBulk(_)
        | GameCommand::Unknown { .. } => {}
    }
}

/// `POS`: the client's position and state; the own one neutralises TeamSpeak's listener, any
/// other recomputes that client's mute and neutralises its 3D position.
fn apply_position(runtime: &PluginRuntime, update: &PositionUpdate) {
    let Some((server, state)) = runtime.current_server() else {
        return;
    };
    let settings = runtime.settings();
    let now = Instant::now();
    let (applied, mute) = {
        let mut state = state.lock();
        let Some(applied) = state.session.apply_position(update, now) else {
            return;
        };
        if applied.is_own_client {
            runtime.ensure_every_sender_dsp(&mut state, &settings);
            (applied, None)
        } else {
            let mute = decide_client_mute(&mut state, &settings, applied.client_id, None, now);
            runtime.ensure_sender_dsp(&mut state, &settings, applied.client_id);
            (applied, mute)
        }
    };
    if let Some(ts) = runtime.act() {
        if applied.is_own_client {
            ts.neutralise_listener(server);
        } else {
            ts.neutralise_client(server, host_client_id(applied.client_id));
        }
    }
    runtime.send_mute_changes(server, mute);
}

/// `FREQ`: the own radios and state; the TeamSpeak nickname follows the game name, the original
/// saved once per game.
fn apply_frequencies(runtime: &PluginRuntime, report: &FrequencyReport) {
    let Some((server, state)) = runtime.current_server() else {
        return;
    };
    let rename = {
        let mut state = state.lock();
        let rename = state.session.apply_frequencies(report);
        if rename.is_some() && state.original_nickname.is_none() {
            state.original_nickname = state
                .session
                .clients
                .own()
                .map(|own| own.nickname().to_owned());
        }
        rename
    };
    let (Some(rename), Some(ts)) = (rename, runtime.act()) else {
        return;
    };
    let length = rename.nickname.chars().count();
    if length < MINIMUM_NICKNAME_LENGTH {
        if !runtime
            .has_warned_short_nickname
            .swap(true, std::sync::atomic::Ordering::AcqRel)
        {
            ts.print_to_current_tab(&format!(
                "Your in-game name is too short({length} characters).\nIt needs to be atleast 3 \
                 characters long.\nThis is a Teamspeak limitation, TFAR will not work properly \
                 until you change your ingame name."
            ));
        }
        return;
    }
    if ts.set_own_nickname(server, &rename.nickname) {
        state
            .lock()
            .session
            .clients
            .client_renamed(rename.client_id, &rename.nickname);
    }
}

/// `KILLED`: the client is dead; the mute matrix of the own channel follows.
fn apply_killed(runtime: &PluginRuntime, nickname: &str) {
    let Some((server, state)) = runtime.current_server() else {
        return;
    };
    let ts = runtime.ts();
    let channel_clients = ts
        .own_client_id(server)
        .and_then(|own| ts.channel_of_client(server, own))
        .map(|channel| ts.channel_clients(server, channel))
        .unwrap_or_default();
    let settings = runtime.settings();
    let changes = {
        let mut state = state.lock();
        state.session.apply_killed(nickname);
        decide_killed_mutes(&mut state, &settings, &channel_clients, Instant::now())
    };
    runtime.send_mute_matrix(server, &changes);
}

/// `RadioTwrAdd`: the towers are added, replacing towers of the same id.
fn add_towers(runtime: &PluginRuntime, records: &[TowerRecord]) {
    let Some((_, state)) = runtime.current_server() else {
        return;
    };
    let antennas: Vec<Antenna> = records
        .iter()
        .filter_map(|record| {
            let id = NetId::new(record.net_id.creator, record.net_id.object);
            Antenna::new(id, record.position, record.range)
                .inspect_err(|error| tracing::warn!(%error, "a radio tower was refused"))
                .ok()
        })
        .collect();
    state.lock().antennas.add(antennas);
}
