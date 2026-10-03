//! Unit tests for the service tick, with the time injected, over the recording fake of
//! TeamSpeak.

use std::time::{Duration, Instant};

use super::*;
use crate::fake_team_speak::{
    FakeAction, FakeTeamSpeak, ME, SERVER, core_over, has_dormant_forbidden_action,
};

/// A status block another radio plugin writes into our metadata.
const OTHER_PLUGIN_METADATA: &str =
    "<TFAR>[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y[/B] [I]P:[/I][B]1.0[/B]</TFAR>";

/// The mission handshake with the serious channel `Serious` and its password `pw`.
const VERSION: &str = "VERSION\t0.9.12\tSerious\tpw";

/// The local player's radios and the game's nickname `Sgt Mock`.
const FREQ: &str = "FREQ\t[\"100_alpha|7|0\"]\t[]\t50\ttrue\t20\t7\tSgt Mock\t0\t7\t1\t20\t1\t20";

/// The local player's own position.
const MY_POSITION: &str = "POS\tSgt Mock\t0\t0\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0";

/// `Alpha` thirty metres away, silent.
const ALPHA_POSITION: &str = "POS\tAlpha\t30\t0\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0";

/// Runs service ticks every [`SERVICE_TICK`] after `now` for `span`; returns the last tick's time.
fn tick_for(
    core: &PluginCore,
    schedule: &mut ServiceSchedule,
    now: Instant,
    span: Duration,
) -> Instant {
    let mut tick = now;
    while tick.saturating_duration_since(now) < span {
        tick += SERVICE_TICK;
        core.service_tick(schedule, tick);
    }
    tick
}

fn metadata_writes(actions: &[FakeAction]) -> Vec<&str> {
    actions
        .iter()
        .filter_map(|action| match action {
            FakeAction::Metadata(metadata) => Some(metadata.as_str()),
            _ => None,
        })
        .collect()
}

fn plays_a_sound(actions: &[FakeAction], file: &str) -> bool {
    actions
        .iter()
        .any(|action| matches!(action, FakeAction::PlayWave(path) if path.ends_with(file)))
}

#[test]
fn after_a_game_times_out_the_plugin_is_dormant_for_ten_seconds_of_ticks_until_the_game_resumes() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    let start = Instant::now();
    let mut schedule = ServiceSchedule::new(start);
    for message in [VERSION, FREQ, MY_POSITION, ALPHA_POSITION] {
        core.handle_game_message(message, start);
    }
    assert!(core.is_in_game());
    fake.take_actions();

    let mut now = start;
    while core.is_in_game() {
        now = tick_for(&core, &mut schedule, now, SERVICE_TICK);
        assert!(now.duration_since(start) < Duration::from_secs(5));
    }
    let ended = fake.take_actions();
    assert!(plays_a_sound(&ended, "off.wav"), "{ended:?}");
    assert!(ended.contains(&FakeAction::Nickname("Mock Player".to_owned())));
    let written = metadata_writes(&ended);
    assert_eq!(written.len(), 1, "{ended:?}");
    assert!(
        written
            .iter()
            .all(|metadata| metadata.contains("[I]Play[/I] [B]N[/B]")),
        "{written:?}"
    );
    assert!(core.is_dormant());

    // Another radio plugin writes its own status into our metadata; a dormant plugin leaves it.
    fake.world()
        .clients
        .entry(ME)
        .and_modify(|me| me.metadata = OTHER_PLUGIN_METADATA.to_owned());
    let now = tick_for(&core, &mut schedule, now, Duration::from_secs(10));
    let dormant = fake.take_actions();
    assert!(!has_dormant_forbidden_action(&dormant), "{dormant:?}");
    assert!(
        !dormant
            .iter()
            .any(|action| matches!(action, FakeAction::PlayWave(_))),
        "{dormant:?}"
    );

    core.handle_game_message(FREQ, now);
    assert!(!core.is_dormant());
    assert!(core.is_in_game());
    let resumed = fake.take_actions();
    assert!(plays_a_sound(&resumed, "on.wav"), "{resumed:?}");
    assert!(resumed.contains(&FakeAction::Nickname("Sgt Mock".to_owned())));
    assert!(
        metadata_writes(&resumed)
            .iter()
            .any(|metadata| metadata.contains("[I]Play[/I] [B]Y[/B]")),
        "{resumed:?}"
    );
}

#[test]
fn a_dormant_plugin_that_never_played_writes_nothing_over_ten_seconds_of_ticks() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    let start = Instant::now();
    let mut schedule = ServiceSchedule::new(start);
    core.handle_game_message(VERSION, start);
    fake.take_actions();
    tick_for(&core, &mut schedule, start, Duration::from_secs(10));
    let actions = fake.take_actions();
    assert!(core.is_dormant());
    assert!(!has_dormant_forbidden_action(&actions), "{actions:?}");
}

#[test]
fn a_game_message_during_the_game_end_handling_keeps_the_plugin_awake() {
    let fake = FakeTeamSpeak::with_players(&[]);
    let core = core_over(&fake);
    let start = Instant::now();
    core.handle_game_message(FREQ, start);
    let timed_out = start + Duration::from_secs(5);
    let timeout = core.session().check_timeout(timed_out);
    assert_eq!(timeout.map(|timeout| timeout.ended_a_game), Some(true));
    core.end_game(SERVER, true);
    // The game comes back before the service thread puts the plugin to sleep.
    core.handle_game_message(FREQ, timed_out);
    core.return_to_dormant();
    assert!(!core.is_dormant());
    assert!(core.is_in_game());
}
