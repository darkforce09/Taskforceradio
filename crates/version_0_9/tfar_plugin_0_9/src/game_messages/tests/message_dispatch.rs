//! Unit tests for the game message dispatch, over the recording fake of TeamSpeak.

use std::time::{Duration, Instant};

use tfar_ts3_host::ClientId;

use crate::fake_team_speak::{
    FakeAction, FakeTeamSpeak, LOBBY, SERIOUS, SERVER, core_over, has_dormant_forbidden_action,
    plugin_commands,
};

/// The mission handshake with the serious channel `Serious` and its password `pw`.
const VERSION: &str = "VERSION\t0.9.12\tSerious\tpw";

/// The local player's radios: one short-range radio on `100_alpha`, alive, voice 20 m, and the
/// game's nickname `Sgt Mock`.
const FREQ: &str = "FREQ\t[\"100_alpha|7|0\"]\t[]\t50\ttrue\t20\t7\tSgt Mock\t0\t7\t1\t20\t1\t20";

/// The local player's own position.
const MY_POSITION: &str = "POS\tSgt Mock\t0\t0\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0";

/// `Alpha` thirty metres away, silent.
const ALPHA_POSITION: &str = "POS\tAlpha\t30\t0\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0";

#[test]
fn the_ping_is_answered_with_pong() {
    let fake = FakeTeamSpeak::with_players(&[]);
    let core = core_over(&fake);
    assert_eq!(
        core.handle_game_message("TS_INFO\tPING", Instant::now()),
        "PONG"
    );
}

#[test]
fn unknown_short_and_malformed_messages_get_fail() {
    let fake = FakeTeamSpeak::with_players(&[]);
    let core = core_over(&fake);
    for message in [
        "TS_INFO\tVERSION",
        "TS_INFO\tPING\textra",
        "POS\tplayer",
        "POS\tAlpha\tx\t0\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0",
        "NO_SUCH_COMMAND\t1",
    ] {
        assert_eq!(
            core.handle_game_message(message, Instant::now()),
            "FAIL",
            "{message}"
        );
    }
}

#[test]
fn version_alone_keeps_the_plugin_dormant_and_writes_nothing() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    fake.take_actions();
    assert_eq!(core.handle_game_message(VERSION, Instant::now()), "OK");
    assert!(core.is_dormant());
    assert!(!core.is_in_game());
    let actions = fake.take_actions();
    assert!(!has_dormant_forbidden_action(&actions), "{actions:?}");
    assert!(
        !actions
            .iter()
            .any(|action| matches!(action, FakeAction::PlayWave(_)))
    );
}

#[test]
fn the_first_game_message_starts_the_game_renames_and_moves_into_the_serious_channel() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    let now = Instant::now();
    core.handle_game_message(VERSION, now);
    fake.take_actions();
    assert_eq!(core.handle_game_message(FREQ, now), "OK");
    assert!(!core.is_dormant());
    assert!(core.is_in_game());
    let actions = fake.take_actions();
    assert!(
        actions.contains(&FakeAction::Nickname("Sgt Mock".to_owned())),
        "{actions:?}"
    );
    assert!(actions.iter().any(
        |action| matches!(action, FakeAction::PlayWave(path) if path.ends_with("radio-sounds/on.wav"))
    ));
    assert!(actions.contains(&FakeAction::Move {
        channel: SERIOUS,
        password: "pw".to_owned(),
    }));
    let metadata = actions.iter().find_map(|action| match action {
        FakeAction::Metadata(metadata) => Some(metadata.clone()),
        _ => None,
    });
    assert_eq!(
        metadata.as_deref(),
        Some(
            "<TFAR>[I]Connected[/I] [B]N[/B] [I]Play[/I] [B]Y[/B] [I]P:[/I][B]0.9.12[/B][I] A: [/I][B]0.9.12[/B]</TFAR>"
        )
    );
}

#[test]
fn a_timeout_ends_the_game_moves_back_and_gives_the_original_nickname_back_once() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    let now = Instant::now();
    core.handle_game_message(VERSION, now);
    core.handle_game_message(FREQ, now);
    fake.world().clients.get_mut(&ClientId(1)).unwrap().channel = SERIOUS;
    fake.take_actions();
    core.check_game_timeout(SERVER, now + Duration::from_secs(5));
    assert!(!core.is_in_game());
    let actions = fake.take_actions();
    assert!(actions.iter().any(
        |action| matches!(action, FakeAction::PlayWave(path) if path.ends_with("radio-sounds/off.wav"))
    ));
    assert!(actions.contains(&FakeAction::Move {
        channel: LOBBY,
        password: String::new(),
    }));
    assert!(
        actions.contains(&FakeAction::Nickname("Mock Player".to_owned())),
        "{actions:?}"
    );
    core.check_game_timeout(SERVER, now + Duration::from_secs(10));
    let later = fake.take_actions();
    assert!(
        !later
            .iter()
            .any(|action| matches!(action, FakeAction::Nickname(_)))
    );
}

#[test]
fn a_silent_player_out_of_voice_range_in_serious_mode_is_muted_once() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    fake.move_everyone(SERIOUS);
    let core = core_over(&fake);
    let now = Instant::now();
    for message in [VERSION, FREQ, MY_POSITION] {
        core.handle_game_message(message, now);
    }
    fake.take_actions();
    assert_eq!(
        core.handle_game_message(ALPHA_POSITION, now),
        "NOT_SPEAKING"
    );
    let actions = fake.take_actions();
    assert!(
        actions.contains(&FakeAction::Mute(vec![ClientId(2)])),
        "{actions:?}"
    );
    assert!(actions.contains(&FakeAction::CentreClient(ClientId(2))));
    core.handle_game_message(ALPHA_POSITION, now);
    assert!(
        !fake
            .take_actions()
            .iter()
            .any(|action| matches!(action, FakeAction::Mute(_)))
    );
}

#[test]
fn outside_serious_mode_nobody_is_muted_or_unmuted() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    let now = Instant::now();
    for message in [VERSION, FREQ, MY_POSITION, ALPHA_POSITION, ALPHA_POSITION] {
        core.handle_game_message(message, now);
    }
    let actions = fake.take_actions();
    assert!(
        !actions
            .iter()
            .any(|action| matches!(action, FakeAction::Mute(_) | FakeAction::Unmute(_)))
    );
}

#[test]
fn a_refused_mute_is_forgotten_and_asked_again() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    fake.move_everyone(SERIOUS);
    fake.world().refuses_mutes = true;
    let core = core_over(&fake);
    let now = Instant::now();
    for message in [VERSION, FREQ, MY_POSITION, ALPHA_POSITION, ALPHA_POSITION] {
        core.handle_game_message(message, now);
    }
    let mutes = fake
        .take_actions()
        .into_iter()
        .filter(|action| matches!(action, FakeAction::Mute(_)))
        .count();
    assert_eq!(mutes, 2);
}

#[test]
fn ts_info_names_the_server_and_our_channel() {
    let fake = FakeTeamSpeak::with_players(&[]);
    let core = core_over(&fake);
    assert_eq!(
        core.handle_game_message("TS_INFO\tSERVER", Instant::now()),
        "Fake Server"
    );
    assert_eq!(
        core.handle_game_message("TS_INFO\tCHANNEL", Instant::now()),
        "Lobby"
    );
}

#[test]
fn killed_with_the_two_fields_arma_sends_replies_done() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    assert_eq!(
        core.handle_game_message("KILLED\tAlpha", Instant::now()),
        "DONE"
    );
}

#[test]
fn is_speaking_follows_the_teamspeak_talking_flag() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    assert_eq!(
        core.handle_game_message("IS_SPEAKING\tAlpha", Instant::now()),
        "NOT_SPEAKING"
    );
    fake.world()
        .clients
        .get_mut(&ClientId(2))
        .unwrap()
        .is_talking = true;
    assert_eq!(
        core.handle_game_message("IS_SPEAKING\tAlpha", Instant::now()),
        "SPEAKING"
    );
    assert_eq!(
        core.handle_game_message("IS_SPEAKING\tNobody", Instant::now()),
        "NOT_SPEAKING"
    );
}

#[test]
fn release_all_tangents_is_forwarded_to_the_channel_and_replies_ok() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    fake.take_actions();
    assert_eq!(
        core.handle_game_message("RELEASE_ALL_TANGENTS\tAlpha", Instant::now()),
        "OK"
    );
    assert_eq!(
        plugin_commands(&fake.take_actions()),
        vec!["RELEASE_ALL_TANGENTS\tAlpha".to_owned()]
    );
}

#[test]
fn a_game_started_before_the_handshake_still_moves_into_the_serious_channel_once() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    let now = Instant::now();
    assert_eq!(core.handle_game_message("TS_INFO\tPING", now), "PONG");
    assert!(core.is_in_game());
    fake.take_actions();
    core.handle_game_message(VERSION, now);
    let moves = |actions: &[FakeAction]| {
        actions
            .iter()
            .filter(|action| matches!(action, FakeAction::Move { .. }))
            .count()
    };
    let actions = fake.take_actions();
    assert_eq!(moves(&actions), 1);
    assert!(actions.iter().any(|action| {
        matches!(action, FakeAction::Metadata(metadata) if metadata.contains("A: [/I][B]0.9.12[/B]"))
    }));
    core.handle_game_message(VERSION, now);
    assert_eq!(moves(&fake.take_actions()), 0);
}
