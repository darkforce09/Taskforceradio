//! Unit tests for the incoming plugin commands, over the recording fake of TeamSpeak.

use std::time::Instant;

use tfar_ts3_host::{ClientId, Invoker, PluginCommand};

use crate::fake_team_speak::{
    FakeAction, FakeTeamSpeak, SERIOUS, SERVER, core_over, has_dormant_forbidden_action,
};
use crate::plugin_core::PluginCore;

const GAME_MESSAGES: [&str; 4] = [
    "VERSION\t0.9.12\tSerious\tpw",
    "FREQ\t[\"100_alpha|7|0\"]\t[]\t50\ttrue\t20\t7\tMock Player\t0\t7\t1\t20\t1\t20",
    "POS\tMock Player\t0\t0\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0",
    "POS\tAlpha\t30\t0\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0",
];

/// Delivers `text` as a plugin command from `Alpha` (client 2) of the plugin `plugin_name`.
fn deliver(core: &PluginCore, plugin_name: &str, text: &str) {
    core.handle_plugin_command(
        &PluginCommand {
            server: SERVER,
            plugin_name,
            command: text,
            invoker: Invoker {
                id: ClientId(2),
                name: "Alpha",
                unique_identifier: "alpha=",
            },
        },
        Instant::now(),
    );
}

#[test]
fn a_remote_press_heard_on_my_radio_unmutes_the_sender_in_serious_mode() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    fake.move_everyone(SERIOUS);
    let core = core_over(&fake);
    for message in GAME_MESSAGES {
        core.handle_game_message(message, Instant::now());
    }
    assert!(
        fake.take_actions()
            .contains(&FakeAction::Mute(vec![ClientId(2)]))
    );
    deliver(
        &core,
        "task_force_radio_linux_amd64",
        "TANGENT\tPRESSED\t100_alpha\t5000\tdigital\tAlpha",
    );
    assert!(
        fake.take_actions()
            .contains(&FakeAction::Unmute(vec![ClientId(2)]))
    );
}

#[test]
fn a_command_of_a_foreign_plugin_is_ignored() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    fake.move_everyone(SERIOUS);
    let core = core_over(&fake);
    for message in GAME_MESSAGES {
        core.handle_game_message(message, Instant::now());
    }
    fake.take_actions();
    deliver(
        &core,
        "other_plugin",
        "TANGENT\tPRESSED\t100_alpha\t5000\tdigital\tAlpha",
    );
    assert_eq!(fake.take_actions(), Vec::<FakeAction>::new());
}

#[test]
fn a_dormant_plugin_takes_in_remote_commands_without_any_request() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    fake.take_actions();
    deliver(&core, "task_force_radio_win64", "VOLUME\tAlpha\t40\ttrue");
    deliver(
        &core,
        "task_force_radio_win64",
        "TANGENT\tPRESSED\t100_alpha\t5000\tdigital\tAlpha",
    );
    let actions = fake.take_actions();
    assert!(!has_dormant_forbidden_action(&actions), "{actions:?}");
    let voice_volume = core
        .with_state(SERVER, |state| {
            state.radio.client("Alpha").map(|alpha| alpha.voice_volume)
        })
        .flatten();
    assert_eq!(voice_volume, Some(40));
}
