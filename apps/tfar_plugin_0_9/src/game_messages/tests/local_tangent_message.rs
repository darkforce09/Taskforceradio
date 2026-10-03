//! Unit tests for the local push-to-talk messages, over the recording fake of TeamSpeak.

use std::sync::Arc;
use std::time::{Duration, Instant};

use super::*;
use crate::fake_team_speak::{FakeAction, FakeTeamSpeak, core_over, plugin_commands};
use crate::plugin_core::PluginCore;

const PRESS: &str = "TANGENT\tPRESSED\t100_alpha\t5000\tdigital";
const RELEASE: &str = "TANGENT\tRELEASED\t100_alpha\t5000\tdigital";

/// Runs the release thread of `core` until `wait` has passed, then stops it.
fn run_release_thread_for(core: &Arc<PluginCore>, wait: Duration) {
    let worker = Arc::clone(core);
    let thread = std::thread::spawn(move || {
        worker
            .push_to_talk()
            .run_release_thread(worker.team_speak());
    });
    std::thread::sleep(wait);
    core.push_to_talk().stop();
    thread.join().unwrap();
}

#[test]
fn the_tangent_families_map_both_ways() {
    for kind in [
        GameTangentKind::ShortRange,
        GameTangentKind::LongRange,
        GameTangentKind::DiverDigital,
    ] {
        assert_eq!(game_tangent_kind(command_tangent_kind(kind)), kind);
    }
}

#[test]
fn a_press_plays_the_local_start_switches_the_microphone_on_and_broadcasts() {
    let fake = FakeTeamSpeak::with_players(&[]);
    let core = core_over(&fake);
    fake.take_actions();
    assert_eq!(core.handle_game_message(PRESS, Instant::now()), "OK");
    let actions = fake.take_actions();
    assert!(actions.iter().any(
        |action| matches!(action, FakeAction::PlayWave(path) if path.ends_with("radio-sounds/sw/local_start.wav"))
    ));
    assert!(actions.contains(&FakeAction::InputActive(true)));
    assert_eq!(
        plugin_commands(&actions),
        vec!["TANGENT\tPRESSED\t100_alpha\t5000\tdigital\tMock Player".to_owned()]
    );
}

#[test]
fn a_repeated_press_sends_nothing() {
    let fake = FakeTeamSpeak::with_players(&[]);
    let core = core_over(&fake);
    core.handle_game_message(PRESS, Instant::now());
    fake.take_actions();
    core.handle_game_message(PRESS, Instant::now());
    assert_eq!(fake.take_actions(), Vec::<FakeAction>::new());
}

#[test]
fn a_tangent_without_a_subtype_broadcasts_six_fields_and_plays_no_sound() {
    let fake = FakeTeamSpeak::with_players(&[]);
    let core = core_over(&fake);
    fake.take_actions();
    core.handle_game_message("TANGENT_LR\tPRESSED\t50_lr\t20000", Instant::now());
    let actions = fake.take_actions();
    assert!(!actions.iter().any(
        |action| matches!(action, FakeAction::PlayWave(path) if path.to_string_lossy().contains("local_"))
    ));
    assert_eq!(
        plugin_commands(&actions),
        vec!["TANGENT_LR\tPRESSED\t50_lr\t20000\t\tMock Player".to_owned()]
    );
}

#[test]
fn a_release_switches_the_microphone_off_and_broadcasts_after_the_delay() {
    let fake = FakeTeamSpeak::with_players(&[]);
    let core = Arc::new(core_over(&fake));
    core.handle_game_message(PRESS, Instant::now());
    core.handle_game_message(RELEASE, Instant::now());
    let actions = fake.take_actions();
    assert!(actions.iter().any(
        |action| matches!(action, FakeAction::PlayWave(path) if path.ends_with("radio-sounds/sw/local_end.wav"))
    ));
    run_release_thread_for(&core, Duration::from_millis(50));
    let released = fake.take_actions();
    assert_eq!(
        released,
        vec![
            FakeAction::InputActive(false),
            FakeAction::PluginCommand(
                "TANGENT\tRELEASED\t100_alpha\t5000\tdigital\tMock Player".to_owned()
            ),
        ]
    );
}

#[test]
fn voice_activation_is_switched_off_for_the_press_and_back_on_after_the_release() {
    let fake = FakeTeamSpeak::with_players(&[]);
    fake.world().is_voice_activation_on = true;
    let core = Arc::new(core_over(&fake));
    core.handle_game_message(PRESS, Instant::now());
    core.handle_game_message(RELEASE, Instant::now());
    run_release_thread_for(&core, Duration::from_millis(50));
    let actions = fake.take_actions();
    let switches: Vec<_> = actions
        .iter()
        .filter(|action| {
            matches!(
                action,
                FakeAction::VoiceActivation(_) | FakeAction::InputActive(_)
            )
        })
        .cloned()
        .collect();
    assert_eq!(
        switches,
        vec![
            FakeAction::VoiceActivation(false),
            FakeAction::InputActive(true),
            FakeAction::VoiceActivation(true),
            FakeAction::InputActive(true),
        ]
    );
}
