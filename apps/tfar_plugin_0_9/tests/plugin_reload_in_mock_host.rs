//! The built plugin `.so` through TeamSpeak's "Reload": the library stays loaded, TeamSpeak
//! registers a new plugin ID and runs `init` again. After the reload the plugin still finds its
//! folder (no "named no plugin folder" warning), plays and mixes the radio sounds from it, loads
//! the HRIR sphere from it, and sends its plugin commands under the new ID.
//!
//! The scenario runs in a child process of this test binary (`scenario_support::run_in_child`),
//! with its own game link port and plugin folder.

#[cfg(test)]
mod scenario_support;

#[cfg(test)]
mod reload {
    use std::time::Duration;

    use tfar_ts3_host::{ChannelId, ClientId};
    use tfar_ts3_mock::{HostCall, MOCK_PLUGIN_ID};

    use super::scenario_support::{
        PLAYING_METADATA, TFAR_PLUGIN_NAME, connect, game_link, load_plugin, plugin_folder,
        run_in_child, send, wait_for_calls, wait_for_calls_within,
    };

    /// The plugin ID TeamSpeak registers on the reload.
    const RELOADED_PLUGIN_ID: &str = "reloaded_plugin_id";

    /// The local player's radios: one short-range radio on `100_alpha`, and the game's nickname.
    const FREQ: &str =
        "FREQ\t[\"100_alpha|7|0\"]\t[]\t50\ttrue\t20\t7\tSgt Mock\t0\t7\t1\t20\t1\t20";

    /// The line the plugin logs when it found no plugin folder.
    const NO_FOLDER_WARNING: &str = "TeamSpeak named no plugin folder";

    #[test]
    fn after_a_reload_the_plugin_finds_its_folder_and_sends_under_the_new_id() {
        run_in_child(
            "reload::after_a_reload_the_plugin_finds_its_folder_and_sends_under_the_new_id",
            reload_scenario,
        );
    }

    /// The log lines of `calls`.
    fn log_lines(calls: &[HostCall]) -> Vec<String> {
        calls
            .iter()
            .filter_map(|call| match call {
                HostCall::LogMessage { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    /// How many log lines of `calls` name the plugin folder TeamSpeak gave.
    fn folder_lines(calls: &[HostCall]) -> usize {
        log_lines(calls)
            .iter()
            .filter(|line| line.contains("the plugin folder from getPluginPath"))
            .count()
    }

    /// The chat lines of `calls`.
    fn chat_lines(calls: &[HostCall]) -> Vec<String> {
        calls
            .iter()
            .filter_map(|call| match call {
                HostCall::PrintMessageToCurrentTab { text } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    fn reload_scenario() {
        let folder = plugin_folder();
        let mut host = load_plugin(&folder);
        assert_eq!(host.registered_plugin_id(), MOCK_PLUGIN_ID);
        host.reload(RELOADED_PLUGIN_ID).unwrap();
        assert_eq!(host.registered_plugin_id(), RELOADED_PLUGIN_ID);
        let after_reload = wait_for_calls_within(&host, Duration::from_secs(2), |calls| {
            folder_lines(calls) == 2
        });
        let logged = log_lines(&after_reload);
        assert_eq!(
            folder_lines(&after_reload),
            2,
            "the first and the reloaded init name the folder from getPluginPath: {logged:#?}"
        );
        assert!(
            !logged.iter().any(|line| line.contains(NO_FOLDER_WARNING)),
            "{logged:#?}"
        );
        let chat = chat_lines(&after_reload);
        assert!(
            !chat.iter().any(|line| line.contains("HRIR sphere")),
            "the HRIR sphere loads from the plugin folder after the reload: {chat:#?}"
        );

        let server = connect(&host);
        let alpha = host.client_joins("Alpha", ChannelId(1)).unwrap();
        host.set_client_metadata(alpha, PLAYING_METADATA).unwrap();
        let link = game_link();
        assert_eq!(send(&link, "TS_INFO\tPING"), "PONG");
        assert_eq!(send(&link, "VERSION\t0.9.12\tSerious\tpw"), "OK");
        assert_eq!(send(&link, FREQ), "OK");
        let game_start = host.take_calls();
        let on_sound = folder.join("radio-sounds/on.wav");
        assert!(
            game_start.iter().any(|call| matches!(
                call,
                HostCall::PlayWaveFile { server: played_on, path }
                    if *played_on == server && *path == on_sound
            )),
            "the game start sound plays from the plugin folder: {game_start:#?}"
        );

        assert_eq!(
            send(&link, "TANGENT\tPRESSED\t100_alpha\t5000\tdigital"),
            "OK"
        );
        let pressed = wait_for_calls(&host, |calls| {
            calls
                .iter()
                .any(|call| matches!(call, HostCall::SendPluginCommand { .. }))
        });
        let senders: Vec<&str> = pressed
            .iter()
            .filter_map(|call| match call {
                HostCall::SendPluginCommand { plugin_id, .. } => Some(plugin_id.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(senders, vec![RELOADED_PLUGIN_ID], "{pressed:#?}");
        assert_eq!(
            send(&link, "TANGENT\tRELEASED\t100_alpha\t5000\tdigital"),
            "OK"
        );
        for client in [ClientId(1), alpha] {
            host.client_moves(client, ChannelId(2)).unwrap();
        }
        for message in [
            "POS\tSgt Mock\t0\t0\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0",
            "POS\tAlpha\t3\t4\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0",
        ] {
            assert_eq!(send(&link, message), "NOT_SPEAKING");
        }
        host.take_calls();

        host.deliver_plugin_command(
            alpha,
            TFAR_PLUGIN_NAME,
            "TANGENT\tPRESSED\t100_alpha\t5000\tdigital\tAlpha",
        )
        .unwrap();
        let mixed = host.mixed_playback(&[0; 960 * 2], 2, 0).unwrap();
        assert!(
            mixed.samples.iter().any(|sample| *sample != 0),
            "the remote start sound from the plugin folder is mixed in"
        );

        host.shutdown();
        let logged = log_lines(&host.calls());
        assert!(
            !logged.iter().any(|line| line.contains(NO_FOLDER_WARNING)),
            "{logged:#?}"
        );
        assert!(host.leak_report().is_clean());
        let _ = std::fs::remove_dir_all(folder);
    }
}
