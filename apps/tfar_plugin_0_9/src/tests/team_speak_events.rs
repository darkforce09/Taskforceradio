//! Unit tests for the TeamSpeak event handling.

use tfar_ts3_host::{ChannelId, ClientId, ClientMove, ErrorDisplay, ServerError, Visibility};

use super::*;
use crate::fake_team_speak::{FakeAction, FakeTeamSpeak, PLAYING_METADATA, SERVER, core_over};

fn error_with_return_code(return_code: &str) -> ServerError<'_> {
    ServerError {
        server: SERVER,
        message: "error",
        error: 1,
        return_code,
        extra_message: "",
    }
}

#[test]
fn a_server_error_with_a_return_code_is_handled_and_one_without_is_shown() {
    assert_eq!(
        server_error_display(&error_with_return_code("tfar")),
        ErrorDisplay::Suppress
    );
    assert_eq!(
        server_error_display(&error_with_return_code("")),
        ErrorDisplay::Show
    );
}

#[test]
fn the_established_connection_neutralises_teamspeaks_3d_sound() {
    let fake = FakeTeamSpeak::with_players(&[]);
    let core = core_over(&fake);
    fake.take_actions();
    core.handle_connect_status(SERVER, ConnectStatus::ConnectionEstablished);
    assert_eq!(fake.take_actions(), vec![FakeAction::Neutralise3d]);
    assert!(core.connection(SERVER).is_some());
    core.handle_connect_status(SERVER, ConnectStatus::Disconnected);
    assert!(core.connection(SERVER).is_none());
}

#[test]
fn a_client_leaving_the_server_is_forgotten() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    let alpha = crate::channel_snapshot::state_client_id(ClientId(2));
    core.with_state(SERVER, |state| {
        let _ = state.mutes.decide(alpha, true);
    });
    fake.world().clients.remove(&ClientId(2));
    core.handle_client_move(&ClientMove {
        server: SERVER,
        client: ClientId(2),
        old_channel: ChannelId(1),
        new_channel: ChannelId(0),
        visibility: Visibility::Leave,
        message: "leaving",
    });
    let state = core
        .with_state(SERVER, |state| {
            (
                state.mutes.is_muted(alpha),
                state.facts.client(alpha).is_some(),
            )
        })
        .unwrap();
    assert_eq!(state, (false, false));
}

#[test]
fn the_info_panel_shows_a_clients_status_block() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    let text = core.info_text(SERVER, InfoItem::Client(ClientId(2)));
    assert_eq!(
        text.as_deref(),
        PLAYING_METADATA
            .strip_prefix("<TFAR>")
            .and_then(|rest| rest.strip_suffix("</TFAR>"))
    );
    assert_eq!(core.info_text(SERVER, InfoItem::Server), None);
}

#[test]
fn the_talking_flag_is_broadcast_as_volume_only_in_game() {
    let fake = FakeTeamSpeak::with_players(&[]);
    let core = core_over(&fake);
    let talking_flag = i32::try_from(ClientProperty::FlagTalking.flag()).unwrap();
    core.handle_own_variable_update(SERVER, talking_flag, "1");
    assert_eq!(fake.take_actions(), Vec::<FakeAction>::new());
    core.handle_game_message(
        "FREQ\t[]\t[]\t\ttrue\t20\t7\tMock Player\t0\t7\t1\t20\t1\t20",
        std::time::Instant::now(),
    );
    fake.take_actions();
    core.handle_own_variable_update(SERVER, talking_flag, "1");
    assert_eq!(
        fake.take_actions(),
        vec![FakeAction::PluginCommand(
            "VOLUME\tMock Player\t20\ttrue".to_owned()
        )]
    );
}

/// A core over `fake` with the HRIR sphere of `assets/hrir/`, so senders get direction sources.
fn core_with_sphere(fake: &std::sync::Arc<FakeTeamSpeak>) -> PluginCore {
    let sphere =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/hrir/IRC_1002_C.bin");
    let core = PluginCore::new(
        std::sync::Arc::clone(fake) as std::sync::Arc<dyn crate::team_speak::TeamSpeak>,
        crate::plugin_core::PluginResources {
            plugin_folder: crate::fake_team_speak::missing_plugin_folder(),
            hrir: Some(tfar_hrtf::HrirLibrary::load(&sphere).unwrap()),
            push_to_talk_delay: std::time::Duration::ZERO,
        },
        std::time::Instant::now(),
    );
    core.add_connection(SERVER);
    core
}

#[test]
fn a_client_who_starts_talking_has_every_direction_source_reset() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_with_sphere(&fake);
    let now = std::time::Instant::now();
    for message in [
        "FREQ\t[\"100_alpha|7|0\"]\t[]\t50\ttrue\t20\t7\tMock Player\t0\t7\t1\t20\t1\t20",
        "POS\tMock Player\t0\t0\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0",
        "POS\tAlpha\t3\t4\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0",
    ] {
        core.handle_game_message(message, now);
    }
    // Alpha's voice reaches 20 m, so Alpha at 5 m speaks to us directly.
    core.handle_plugin_command(
        &tfar_ts3_host::PluginCommand {
            server: SERVER,
            plugin_name: "task_force_radio_linux_amd64",
            command: "VOLUME\tAlpha\t20\ttrue",
            invoker: tfar_ts3_host::Invoker {
                id: ClientId(2),
                name: "Alpha",
                unique_identifier: "alpha=",
            },
        },
        now,
    );
    let alpha = state_client_id(ClientId(2));
    let reset = core.with_state(SERVER, |state| state.reset_direction_sources_of(alpha));
    assert!(reset.is_some_and(|reset| reset >= 1), "{reset:?}");
    let nobody = state_client_id(ClientId(9));
    assert_eq!(
        core.with_state(SERVER, |state| state.reset_direction_sources_of(nobody)),
        Some(0)
    );
    fake.take_actions();
    core.handle_talk_status_change(SERVER, TalkStatus::Talking, ClientId(2));
    core.handle_talk_status_change(SERVER, TalkStatus::NotTalking, ClientId(2));
    assert_eq!(fake.take_actions(), Vec::<FakeAction>::new());
}
