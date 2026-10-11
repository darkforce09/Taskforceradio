//! The built 1.0 plugin `.so` through TeamSpeak's "Reload": the library stays loaded, TeamSpeak
//! registers a new plugin ID and runs `init` again. After the reload the plugin still finds its
//! folder from `getPluginPath`, loads the HRIR sphere and the radio sounds from it (the game start
//! sound plays from it, a remote start sound is mixed in), and sends its plugin commands under
//! the new ID.
//!
//! The plugin reads its port from the process environment, so the test re-runs its own binary as
//! a child with `TFAR_GAME_LINK_PORT` set to a free port, the update check off and no X11
//! display; the child loads the plugin and runs the scenario.

#[cfg(test)]
mod reload {
    use std::net::TcpListener;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{Duration, Instant};

    use tfar_game_link_client::prelude::{ClientConfig, Flavour, GameLinkClient, SyncOutcome};
    use tfar_ts3_host::{ChannelId, ClientId};
    use tfar_ts3_mock::{
        HostCall, HostOptions, MOCK_PLUGIN_ID, MockHost, ServerSetup, locate_cdylib,
    };

    /// Set in the child's environment to the name of the scenario it runs.
    const CHILD_MARKER: &str = "TFAR_PLUGIN_1_0_RELOAD_CHILD";

    /// The name of the one test, which the parent passes to the child's test harness.
    const TEST_NAME: &str =
        "reload::after_a_reload_the_plugin_finds_its_folder_and_sends_under_the_new_id";

    /// How long the scenario waits for an effect of an async message.
    const EFFECT_TIMEOUT: Duration = Duration::from_secs(3);

    /// The plugin ID TeamSpeak registers on the reload.
    const RELOADED_PLUGIN_ID: &str = "reloaded_plugin_id";

    /// The line `plugin_folder` logs when `getPluginPath` answered.
    const FOLDER_FROM_TEAMSPEAK: &str = "the plugin folder from getPluginPath";

    /// The plugin name other TFAR clients send plugin commands with.
    const REMOTE_PLUGIN_NAME: &str = "TFAR_linux_amd64";

    /// The metadata of a client that runs TFAR and plays.
    const PLAYING_METADATA: &str = "<TFAR>Connected to Game: [B]Yes[/B]\nPlaying: [B]Yes[/B]\nPlugin \
                                version: [B]1.0.0.1[/B]\nAddon version: [B]1.0[/B]\n</TFAR>";

    #[test]
    fn after_a_reload_the_plugin_finds_its_folder_and_sends_under_the_new_id() {
        if std::env::var(CHILD_MARKER).as_deref() == Ok(TEST_NAME) {
            reload_scenario();
            return;
        }
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let status = Command::new(std::env::current_exe().unwrap())
            .args([TEST_NAME, "--exact", "--nocapture", "--test-threads=1"])
            .env(CHILD_MARKER, TEST_NAME)
            .env("TFAR_GAME_LINK_PORT", port.to_string())
            .env("TFAR_SKIP_UPDATE_CHECK", "1")
            .env_remove("DISPLAY")
            .status()
            .unwrap();
        assert!(
            status.success(),
            "the scenario failed in the child: {status}"
        );
    }

    /// A plugins folder holding the 1.0 `radio-sounds/` from `reference/` and
    /// `tfar_hrir/IRC_1002_C.bin` from `assets/hrir/`, as `cargo xtask dist` lays them out.
    fn plugin_folder() -> PathBuf {
        let folder = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("tfar-plugin-1-0-reload-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        copy_folder(
            &root.join("reference/tfar_1_0/ts/radio-sounds"),
            &folder.join("radio-sounds"),
        );
        std::fs::create_dir_all(folder.join("tfar_hrir")).unwrap();
        std::fs::copy(
            root.join("assets/hrir/IRC_1002_C.bin"),
            folder.join("tfar_hrir/IRC_1002_C.bin"),
        )
        .unwrap();
        folder
    }

    fn copy_folder(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_folder(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    /// A game link client on the scenario's port with a live session.
    fn game_client() -> GameLinkClient {
        let port: u16 = std::env::var("TFAR_GAME_LINK_PORT")
            .unwrap()
            .parse()
            .unwrap();
        let client = GameLinkClient::new(ClientConfig::new(Flavour::V1_0, port));
        assert_eq!(
            client.sync_request(b"TS_INFO\tPING"),
            SyncOutcome::Reply(b"PONG".to_vec())
        );
        client
    }

    /// The sync reply to `text` as text.
    fn sync(client: &GameLinkClient, text: &str) -> String {
        match client.sync_request(text.as_bytes()) {
            SyncOutcome::Reply(reply) => String::from_utf8(reply).unwrap(),
            other => panic!("no reply to {text:?}: {other:?}"),
        }
    }

    /// Sends the async message `text` (without the `~`).
    fn send(client: &GameLinkClient, text: &str) {
        client.send_async(text.as_bytes()).unwrap();
    }

    /// Waits until `condition` holds; panics with `what` after the timeout.
    fn wait_until(what: &str, condition: impl Fn() -> bool) {
        let started = Instant::now();
        while started.elapsed() < EFFECT_TIMEOUT {
            if condition() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("timed out waiting for {what}");
    }

    /// Waits until `found` holds for the host's calls; panics with `what` after the timeout.
    fn wait_for(host: &MockHost, what: &str, found: impl Fn(&[HostCall]) -> bool) {
        let started = Instant::now();
        while started.elapsed() < EFFECT_TIMEOUT {
            if found(&host.calls()) {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("timed out waiting for {what}; calls: {:#?}", host.calls());
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
            .filter(|line| line.contains(FOLDER_FROM_TEAMSPEAK))
            .count()
    }

    /// The plugin IDs the plugin sent its commands under, in order.
    fn command_plugin_ids(calls: &[HostCall]) -> Vec<String> {
        calls
            .iter()
            .filter_map(|call| match call {
                HostCall::SendPluginCommand { plugin_id, .. } => Some(plugin_id.clone()),
                _ => None,
            })
            .collect()
    }

    /// The root mean square of `samples`.
    fn rms(samples: &[i16]) -> f64 {
        #[expect(clippy::cast_precision_loss)]
        let count = samples.len().max(1) as f64;
        (samples
            .iter()
            .map(|sample| f64::from(*sample).powi(2))
            .sum::<f64>()
            / count)
            .sqrt()
    }

    fn reload_scenario() {
        let folder = plugin_folder();
        let library = locate_cdylib("tfar_plugin_1_0").expect("libtfar_plugin_1_0.so is built");
        let options = HostOptions {
            plugin_path: Some(folder.clone()),
            config_path: None,
        };
        let mut host = MockHost::load_with_options(&library, &options).unwrap();
        assert_eq!(host.registered_plugin_id(), MOCK_PLUGIN_ID);
        host.reload(RELOADED_PLUGIN_ID).unwrap();
        assert_eq!(host.registered_plugin_id(), RELOADED_PLUGIN_ID);
        check_the_reloaded_init(&host);

        host.connect_server(&ServerSetup {
            own_nickname: "Alpha TS".to_owned(),
            channel_names: vec!["Lobby".to_owned(), "TaskForceRadio".to_owned()],
            ..ServerSetup::default()
        })
        .unwrap();
        let bravo = host.client_joins("Bravo", ChannelId(2)).unwrap();
        host.set_client_metadata(bravo, PLAYING_METADATA).unwrap();
        let client = game_client();
        client.send_focus(true).unwrap();
        start_the_game(&host, &client, &folder);
        transmit_under_the_new_id(&host, &client);
        host.deliver_plugin_command(
            bravo,
            REMOTE_PLUGIN_NAME,
            "TANGENT\tPRESSED\t30.0\t20000\tdigital\ttf_anprc152_7\t1\t0\tBravo",
        )
        .unwrap();
        let mixed = host.mixed_playback(&[0_i16; 960], 2, 0).unwrap();
        assert!(
            rms(&mixed.samples) > 100.0,
            "the remote start sound from the plugin folder is mixed in"
        );

        host.shutdown();
        let logged = log_lines(&host.calls());
        assert!(
            !logged.iter().any(
                |line| line.contains("getPluginPath") && line.contains("TeamSpeak call failed")
            ),
            "{logged:#?}"
        );
        assert!(host.leak_report().is_clean());
        let _ = std::fs::remove_dir_all(folder);
    }

    /// The reloaded `init` names the folder from `getPluginPath` (as the first one did) and finds
    /// the HRIR sphere in it.
    fn check_the_reloaded_init(host: &MockHost) {
        let has_failed_path = |calls: &[HostCall]| {
            log_lines(calls)
                .iter()
                .any(|line| line.contains("TeamSpeak call failed: getPluginPath"))
        };
        let resolved_folders = |calls: &[HostCall]| {
            log_lines(calls)
                .iter()
                .filter(|line| line.contains("the plugin folder from"))
                .count()
        };
        wait_until(
            "both inits resolve the plugin folder, or one fails to",
            || {
                let calls = host.calls();
                resolved_folders(&calls) == 2 || has_failed_path(&calls)
            },
        );
        let calls = host.calls();
        let logged = log_lines(&calls);
        assert!(!has_failed_path(&calls), "{logged:#?}");
        assert_eq!(folder_lines(&calls), 2, "{logged:#?}");
        let chat: Vec<&HostCall> = calls
            .iter()
            .filter(|call| {
                matches!(call, HostCall::PrintMessageToCurrentTab { text }
                    if text.contains("HRIR sphere"))
            })
            .collect();
        assert!(chat.is_empty(), "the HRIR sphere loads: {chat:#?}");
    }

    /// The settings, the move into the serious channel and the game start sound from `folder`.
    fn start_the_game(host: &MockHost, client: &GameLinkClient, folder: &Path) {
        assert_eq!(sync(client, "TS_INFO\tVERSION"), "1.0.0.1");
        wait_until("needcfg at start", || client.needcfg());
        send(
            client,
            "SETCFG\tserious_channelName\tTaskForceRadio\tSTRING",
        );
        send(client, "SETCFG\tserious_channelPassword\t123\tSTRING");
        wait_until("needcfg cleared", || !client.needcfg());
        let on_sound = folder.join("radio-sounds/on.wav");
        wait_for(
            host,
            "the game start sound from the plugin folder",
            |calls| {
                calls.iter().any(
                    |call| matches!(call, HostCall::PlayWaveFile { path, .. } if *path == on_sound),
                )
            },
        );
        host.client_moves(ClientId(1), ChannelId(2)).unwrap();
        send(
            client,
            "FREQ\t[\"30.0|7|0|tf_anprc152\"]\t[]\ttrue\t20\tAlpha\t0\t7\t1\t1\t20",
        );
        for update in [
            "POS\tAlpha\t[0,0,0]\t[0,1,0]\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0\tfalse\tfalse",
            "POS\tBravo\t[300,0,0]\t[0,1,0]\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0\tfalse\tfalse",
        ] {
            assert_eq!(sync(client, update), "00");
        }
    }

    /// A local press broadcasts under the ID TeamSpeak registered on the reload.
    fn transmit_under_the_new_id(host: &MockHost, client: &GameLinkClient) {
        host.take_calls();
        send(
            client,
            "TANGENT\tPRESSED\t30.0\t20000\tdigital\ttf_anprc152_1",
        );
        wait_for(host, "the press broadcast", |calls| {
            !command_plugin_ids(calls).is_empty()
        });
        send(
            client,
            "TANGENT\tRELEASED\t30.0\t20000\tdigital\ttf_anprc152_1",
        );
        wait_for(host, "the release broadcast", |calls| {
            command_plugin_ids(calls).len() >= 2
        });
        let ids = command_plugin_ids(&host.calls());
        assert!(
            ids.iter().all(|id| id == RELOADED_PLUGIN_ID),
            "every plugin command after the reload carries the new ID: {ids:?}"
        );
    }
}
