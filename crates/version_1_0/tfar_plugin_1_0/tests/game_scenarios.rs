//! The built 1.0 plugin `.so` in the mock TeamSpeak host, driven by a real game link client:
//! the full game scenario, the dormant plugin, the return to dormant after a game end, the mission
//! end and the tangent release it cancels, needcfg, radio towers, the debug window without a
//! display, the HRTF restart after an idle gap (never on a talk start), and a shutdown that joins every thread
//! within 2 s.
//!
//! The plugin reads its port and switches from the process environment, so each test re-runs its
//! own binary as a child with `TFAR_GAME_LINK_PORT` set to a free port, the update check off and
//! no X11 display (a scenario may set more variables, such as a shorter service tick or a local
//! proxy); the child loads the plugin and runs one scenario.

#[cfg(test)]
mod scenarios {
    use std::f64::consts::TAU;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{Duration, Instant};

    use tfar_game_link_client::prelude::{ClientConfig, Flavour, GameLinkClient, SyncOutcome};
    use tfar_ts3_host::{ChannelId, ClientId, ClientProperty};
    use tfar_ts3_mock::{
        HostCall, HostOptions, MockHost, SentCommandTarget, ServerSetup, locate_cdylib,
    };

    /// Set in a child's environment to the name of the scenario it runs.
    const CHILD_MARKER: &str = "TFAR_PLUGIN_1_0_SCENARIO_CHILD";

    /// How long a scenario waits for an effect of an async message.
    const EFFECT_TIMEOUT: Duration = Duration::from_secs(3);

    /// The plugin name other TFAR clients send plugin commands with.
    const REMOTE_PLUGIN_NAME: &str = "TFAR_linux_amd64";

    /// The metadata of a client that runs TFAR and plays.
    const PLAYING_METADATA: &str = "<TFAR>Connected to Game: [B]Yes[/B]\nPlaying: [B]Yes[/B]\nPlugin \
                                version: [B]1.0.0.1[/B]\nAddon version: [B]1.0[/B]\n</TFAR>";

    /// Runs `scenario` in a child process of this test binary, or right here inside the child.
    pub(crate) fn run_in_child(test_name: &str, scenario: fn()) {
        run_in_child_with(test_name, scenario, &[]);
    }

    /// [`run_in_child`] with the variables of `environment` set in the child, over the defaults.
    pub(crate) fn run_in_child_with(
        test_name: &str,
        scenario: fn(),
        environment: &[(&str, String)],
    ) {
        if std::env::var(CHILD_MARKER).as_deref() == Ok(test_name) {
            scenario();
            return;
        }
        let status = Command::new(std::env::current_exe().unwrap())
            .args([test_name, "--exact", "--nocapture", "--test-threads=1"])
            .env(CHILD_MARKER, test_name)
            .env("TFAR_GAME_LINK_PORT", free_port().to_string())
            .env("TFAR_SKIP_UPDATE_CHECK", "1")
            .env_remove("DISPLAY")
            .envs(
                environment
                    .iter()
                    .map(|(name, value)| (*name, value.as_str())),
            )
            .status()
            .unwrap();
        assert!(
            status.success(),
            "the scenario failed in the child: {status}"
        );
    }

    /// A port nothing listens on right now.
    fn free_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    }

    /// A 48 kHz stereo 16-bit WAV of `frames` frames of a loud square wave.
    fn wave_bytes(frames: u32) -> Vec<u8> {
        let data_bytes = frames * 4;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_bytes).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&2_u16.to_le_bytes());
        bytes.extend_from_slice(&48_000_u32.to_le_bytes());
        bytes.extend_from_slice(&(48_000_u32 * 4).to_le_bytes());
        bytes.extend_from_slice(&4_u16.to_le_bytes());
        bytes.extend_from_slice(&16_u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_bytes.to_le_bytes());
        for frame in 0..frames {
            let sample: i16 = if frame % 48 < 24 { 12_000 } else { -12_000 };
            bytes.extend_from_slice(&sample.to_le_bytes());
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        bytes
    }

    /// A plugins folder with every radio sound, under the target temp folder.
    fn plugin_folder(name: &str) -> PathBuf {
        let folder = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("tfar-plugin-1-0-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        for kind in ["sw", "lr", "ab"] {
            std::fs::create_dir_all(folder.join("radio-sounds").join(kind)).unwrap();
            for event in ["local_start", "local_end", "remote_start", "remote_end"] {
                let path = folder.join(format!("radio-sounds/{kind}/{event}.wav"));
                std::fs::write(path, wave_bytes(4_800)).unwrap();
            }
        }
        for sound in ["on", "off"] {
            std::fs::write(
                folder.join(format!("radio-sounds/{sound}.wav")),
                wave_bytes(480),
            )
            .unwrap();
        }
        folder
    }

    /// Loads the plugin with the radio sounds of `name`'s folder.
    fn load_plugin(name: &str) -> (MockHost, PathBuf) {
        load_plugin_from(plugin_folder(name))
    }

    /// Loads the plugin with the radio sounds and the HRIR sphere of `assets/hrir/`, so HRTF is
    /// on.
    fn load_plugin_with_hrir(name: &str) -> (MockHost, PathBuf) {
        let folder = plugin_folder(name);
        let sphere =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../assets/hrir/IRC_1002_C.bin");
        std::fs::create_dir_all(folder.join("tfar_hrir")).unwrap();
        std::fs::copy(sphere, folder.join("tfar_hrir/IRC_1002_C.bin")).unwrap();
        load_plugin_from(folder)
    }

    /// Loads the plugin with the plugins folder `folder`.
    fn load_plugin_from(folder: PathBuf) -> (MockHost, PathBuf) {
        let library = locate_cdylib("tfar_plugin_1_0").expect("libtfar_plugin_1_0.so is built");
        let options = HostOptions {
            plugin_path: Some(folder.clone()),
            config_path: None,
        };
        (
            MockHost::load_with_options(&library, &options).unwrap(),
            folder,
        )
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

    /// Waits until `condition` holds.
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

    /// The SDK flag of a client property.
    fn flag(property: ClientProperty) -> usize {
        property.flag()
    }

    /// Whether `calls` hold a plugin command to the current channel that starts with `prefix`.
    fn sent_command(calls: &[HostCall], prefix: &str) -> bool {
        calls.iter().any(|call| {
        matches!(call, HostCall::SendPluginCommand { command, target: SentCommandTarget::CurrentChannel, .. }
            if command.starts_with(prefix))
    })
    }

    /// The values the plugin set for `property` on itself, in order.
    fn own_int_values(calls: &[HostCall], property: ClientProperty) -> Vec<i32> {
        calls
            .iter()
            .filter_map(|call| match call {
                HostCall::SetClientSelfVariableAsInt {
                    flag: set, value, ..
                } if *set == flag(property) => Some(*value),
                _ => None,
            })
            .collect()
    }

    /// The texts the plugin set for `property` on itself, in order.
    fn own_text_values(calls: &[HostCall], property: ClientProperty) -> Vec<String> {
        calls
            .iter()
            .filter_map(|call| match call {
                HostCall::SetClientSelfVariableAsString {
                    flag: set, value, ..
                } if *set == flag(property) => Some(value.clone()),
                _ => None,
            })
            .collect()
    }

    /// Whether the plugin played `sound` (relative, without `.wav`) through TeamSpeak.
    fn played(calls: &[HostCall], folder: &Path, sound: &str) -> bool {
        let expected = folder.join(format!("{sound}.wav"));
        calls
            .iter()
            .any(|call| matches!(call, HostCall::PlayWaveFile { path, .. } if *path == expected))
    }

    /// `frames` mono frames of a sine at `hertz`, amplitude `level`, starting at frame `start`.
    fn sine(hertz: f64, level: f64, start: usize, frames: usize) -> Vec<i16> {
        (start..start + frames)
            .map(|frame| {
                #[expect(clippy::cast_precision_loss)]
                let value = (TAU * hertz * frame as f64 / 48_000.0).sin() * level;
                #[expect(clippy::cast_possible_truncation)]
                let sample = value as i16;
                sample
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

    /// A `POS` message for `nickname` at `position`.
    fn position(nickname: &str, position: [f64; 3]) -> String {
        format!(
            "POS\t{nickname}\t[{},{},{}]\t[0,1,0]\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0\tfalse\tfalse",
            position[0], position[1], position[2]
        )
    }

    /// Connects the server with the serious channel and two other players in it.
    fn connect_three_clients(host: &MockHost) -> (ClientId, ClientId, ClientId) {
        host.connect_server(&ServerSetup {
            own_nickname: "Alpha TS".to_owned(),
            channel_names: vec!["Lobby".to_owned(), "TaskForceRadio".to_owned()],
            ..ServerSetup::default()
        })
        .unwrap();
        let bravo = host.client_joins("Bravo", ChannelId(2)).unwrap();
        let charlie = host.client_joins("Charlie", ChannelId(2)).unwrap();
        host.set_client_metadata(bravo, PLAYING_METADATA).unwrap();
        (ClientId(1), bravo, charlie)
    }

    pub(crate) fn full_scenario() {
        let (mut host, folder) = load_plugin("full");
        let (own, bravo, charlie) = connect_three_clients(&host);
        let client = game_client();
        client.send_focus(true).unwrap();

        start_game_in_serious_channel(&host, &client, own, &folder);
        position_three_clients(&host, &client, charlie);
        transmit_locally(&host, &client);
        receive_remote_radio(&host, bravo, charlie);
        rename_twice(&host, &client);
        shut_down_cleanly(&mut host, charlie);
        let _ = std::fs::remove_dir_all(folder);
    }

    /// The settings, the move into the serious channel and the game start.
    fn start_game_in_serious_channel(
        host: &MockHost,
        client: &GameLinkClient,
        own: ClientId,
        folder: &Path,
    ) {
        assert_eq!(sync(client, "TS_INFO\tVERSION"), "1.0.0.1");
        assert_eq!(sync(client, "TS_INFO\tSERVER"), "Mock Server");
        wait_until("needcfg at start", || client.needcfg());
        send(
            client,
            "SETCFG\tserious_channelName\tTaskForceRadio\tSTRING",
        );
        send(client, "SETCFG\tserious_channelPassword\t123\tSTRING");
        wait_until("needcfg cleared", || !client.needcfg());
        wait_for(host, "the move into the serious channel", |calls| {
            calls.iter().any(|call| {
                matches!(call, HostCall::RequestClientMove { client, channel, .. }
            if *client == own && *channel == ChannelId(2))
            })
        });
        assert!(played(&host.calls(), folder, "radio-sounds/on"));
        host.client_moves(own, ChannelId(2)).unwrap();
    }

    /// The radios, the rename, the positions and the serious-mode mute.
    fn position_three_clients(host: &MockHost, client: &GameLinkClient, charlie: ClientId) {
        send(
            client,
            "FREQ\t[\"30.0|7|0|tf_anprc152\"]\t[]\ttrue\t20\tAlpha\t0\t7\t1\t1\t20",
        );
        wait_for(host, "the rename", |calls| {
            own_text_values(calls, ClientProperty::Nickname) == ["Alpha"]
        });
        // The rename call above is recorded before the plugin takes the new own nickname into its
        // state, and `POS` goes the sync way, so the own `POS` is sent until it is taken as ours.
        wait_until("the own POS taken as the local player's", || {
            assert_eq!(sync(client, &position("Alpha", [0.0, 0.0, 0.0])), "00");
            host.calls().into_iter().any(|call| {
                matches!(
                    call,
                    HostCall::Systemset3DListenerAttributes {
                        position: Some(_),
                        ..
                    }
                )
            })
        });
        for update in [
            position("Bravo", [300.0, 0.0, 0.0]),
            position("Charlie", [900.0, 0.0, 0.0]),
        ] {
            assert_eq!(sync(client, &update), "00");
        }
        wait_for(
            host,
            "Charlie muted out of range in serious mode",
            |calls| {
                calls.iter().any(|call| {
                    matches!(call, HostCall::RequestMuteClients { clients, .. }
            if clients.contains(&charlie))
                })
            },
        );
        let centred_listener = host.calls().into_iter().any(|call| {
            matches!(
                call,
                HostCall::Systemset3DListenerAttributes {
                    position: Some(_),
                    forward: None,
                    up: None,
                    ..
                }
            )
        });
        assert!(
            centred_listener,
            "the own POS centres the listener, its forward and up vectors left null"
        );
        let metadata = own_text_values(&host.calls(), ClientProperty::MetaData);
        assert!(
        metadata.iter().any(|text| text.contains(
            "<TFAR>Connected to Game: [B]Yes[/B]\nPlaying: [B]Yes[/B]\nPlugin version: [B]1.0.0.1[/B]"
        )),
        "{metadata:?}"
    );
    }

    /// The local tangent: microphone on, broadcast, then off again after the release.
    fn transmit_locally(host: &MockHost, client: &GameLinkClient) {
        send(
            client,
            "TANGENT\tPRESSED\t30.0\t20000\tdigital\ttf_anprc152_1",
        );
        wait_for(host, "the press broadcast", |calls| {
            sent_command(
                calls,
                "TANGENT\tPRESSED\t30.0\t20000\tdigital\ttf_anprc152_1\t1\t0\tAlpha",
            )
        });
        send(
            client,
            "TANGENT\tRELEASED\t30.0\t20000\tdigital\ttf_anprc152_1",
        );
        wait_for(host, "the release broadcast", |calls| {
            sent_command(calls, "TANGENT\tRELEASED")
        });
        assert_eq!(
            own_int_values(&host.calls(), ClientProperty::InputDeactivated),
            [0, 1],
            "the microphone is forced on for the press and off after the release"
        );
        release_all_tangents_frees_the_next_press(host, client);
    }

    /// The number of press broadcasts in `calls`.
    fn press_broadcasts(calls: &[HostCall]) -> usize {
        calls
            .iter()
            .filter(|call| {
                matches!(call, HostCall::SendPluginCommand { command, .. }
                    if command.starts_with("TANGENT\tPRESSED"))
            })
            .count()
    }

    /// `RELEASE_ALL_TANGENTS` clears the own tangent, so the next press transmits again.
    fn release_all_tangents_frees_the_next_press(host: &MockHost, client: &GameLinkClient) {
        send(
            client,
            "TANGENT\tPRESSED\t30.0\t20000\tdigital\ttf_anprc152_1",
        );
        wait_for(host, "the second press", |calls| {
            press_broadcasts(calls) == 2
        });
        send(client, "RELEASE_ALL_TANGENTS\tAlpha");
        wait_for(host, "the release-all broadcast", |calls| {
            sent_command(calls, "RELEASE_ALL_TANGENTS\tAlpha")
        });
        send(
            client,
            "TANGENT\tPRESSED\t30.0\t20000\tdigital\ttf_anprc152_1",
        );
        wait_for(host, "a press after the release-all", |calls| {
            press_broadcasts(calls) == 3
        });
        send(
            client,
            "TANGENT\tRELEASED\t30.0\t20000\tdigital\ttf_anprc152_1",
        );
    }

    /// A second game name renames again, but the nickname saved for game end stays the first one.
    fn rename_twice(host: &MockHost, client: &GameLinkClient) {
        send(
            client,
            "FREQ\t[\"30.0|7|0|tf_anprc152\"]\t[]\ttrue\t20\tAlpha Two\t0\t7\t1\t1\t20",
        );
        wait_for(host, "the second rename", |calls| {
            own_text_values(calls, ClientProperty::Nickname) == ["Alpha", "Alpha Two"]
        });
    }

    /// A remote press on the own frequency: the remote sound, then the radio voice.
    fn receive_remote_radio(host: &MockHost, bravo: ClientId, charlie: ClientId) {
        host.deliver_plugin_command(
            bravo,
            REMOTE_PLUGIN_NAME,
            "TANGENT\tPRESSED\t30.0\t20000\tdigital\ttf_anprc152_7\t1\t0\tBravo",
        )
        .unwrap();
        let mixed = host.mixed_playback(&[0_i16; 960], 2, 0).unwrap();
        assert!(
            rms(&mixed.samples) > 100.0,
            "the remote start sound is mixed in"
        );
        assert_eq!(mixed.channel_fill_mask & 3, 3);

        let mut low_in = Vec::new();
        let mut low_out = Vec::new();
        for block in 0..20 {
            let samples = sine(60.0, 8_000.0, block * 480, 480);
            let outcome = host.post_process_voice(bravo, &samples, 1).unwrap();
            if block >= 10 {
                low_in.extend_from_slice(&samples);
                low_out.extend_from_slice(&outcome.samples);
            }
        }
        assert_ne!(low_in, low_out, "the radio voice is processed");
        assert!(
            rms(&low_out) < rms(&low_in) / 4.0,
            "a 60 Hz voice is band-limited away by the radio: {} vs {}",
            rms(&low_out),
            rms(&low_in)
        );
        let plain = sine(1_000.0, 8_000.0, 0, 480);
        let outcome = host.post_process_voice(charlie, &plain, 1).unwrap();
        assert_eq!(
            outcome.samples, plain,
            "a client without TFAR passes through"
        );
    }

    /// Shutdown within 2 s: game end, metadata cleared, mutes lifted, no leak.
    fn shut_down_cleanly(host: &mut MockHost, charlie: ClientId) {
        let shutdown_started = Instant::now();
        host.shutdown();
        assert!(
            shutdown_started.elapsed() < Duration::from_secs(2),
            "shutdown took {:?}",
            shutdown_started.elapsed()
        );
        let calls = host.calls();
        assert_eq!(
            own_text_values(&calls, ClientProperty::Nickname).last(),
            Some(&"Alpha TS".to_owned()),
            "game end restores the nickname from before the first rename"
        );
        assert!(calls.iter().any(
            |call| matches!(call, HostCall::RequestUnmuteClients { clients, .. }
        if clients == &[charlie])
        ));
        let metadata = own_text_values(&calls, ClientProperty::MetaData);
        assert!(
            metadata
                .iter()
                .any(|text| text.contains("Task Force Radio Plugin Disabled"))
        );
        assert!(metadata.last().is_some_and(|text| !text.contains("<TFAR>")));
        let leaks = host.leak_report();
        assert!(
            leaks.is_clean(),
            "the plugin leaked TeamSpeak memory: {leaks:?}"
        );
    }

    pub(crate) fn dormant_plugin() {
        let (mut host, folder) = load_plugin("dormant");
        let (_, bravo, _) = connect_three_clients(&host);
        host.deliver_plugin_command(bravo, REMOTE_PLUGIN_NAME, "VOLUME\tBravo\t20\ttrue")
            .unwrap();
        let samples = sine(1_000.0, 8_000.0, 0, 480);
        assert_eq!(
            host.post_process_voice(bravo, &samples, 1).unwrap().samples,
            samples
        );
        let mixed = host.mixed_playback(&[7_i16; 960], 2, 3).unwrap();
        assert!(mixed.samples.iter().all(|sample| *sample == 7));
        std::thread::sleep(Duration::from_millis(300));
        host.shutdown();
        let acting = host.calls().into_iter().filter(|call| {
            matches!(
                call,
                HostCall::SetClientSelfVariableAsString { .. }
                    | HostCall::SetClientSelfVariableAsInt { .. }
                    | HostCall::RequestMuteClients { .. }
                    | HostCall::RequestUnmuteClients { .. }
                    | HostCall::RequestClientMove { .. }
                    | HostCall::SendPluginCommand { .. }
                    | HostCall::PlayWaveFile { .. }
            )
        });
        assert_eq!(
            acting.count(),
            0,
            "a dormant plugin acts on nothing: {:#?}",
            host.calls()
        );
        assert!(host.leak_report().is_clean());
        let _ = std::fs::remove_dir_all(folder);
    }

    pub(crate) fn mission_end_and_needcfg() {
        let (mut host, folder) = load_plugin("mission");
        host.connect_server(&ServerSetup::default()).unwrap();
        let client = game_client();
        wait_until("needcfg at start", || client.needcfg());
        send(&client, "DFRAME");
        send(&client, "SETCFG\tpluginTimeout\t4\tSCALAR");
        wait_until("needcfg cleared by SETCFG", || !client.needcfg());
        wait_for(&host, "the on sound", |calls| {
            played(calls, &folder, "radio-sounds/on")
        });
        // A tangent let go with a 1.5 s release delay, still waiting when the mission ends.
        send(&client, &frequencies("Alpha"));
        send(&client, "SETCFG\ttangentReleaseDelay\t1500\tSCALAR");
        let tangent = "TANGENT\tPRESSED\t30.0\t20000\tdigital\ttf_anprc152_1";
        send(&client, tangent);
        wait_for(&host, "the press broadcast", |calls| {
            sent_command(calls, "TANGENT\tPRESSED")
        });
        send(&client, &tangent.replace("PRESSED", "RELEASED"));
        let released = Instant::now();
        // The connect transition opens the 500 ms debounce window before the disconnect may follow.
        std::thread::sleep(Duration::from_millis(600));
        send(&client, "MISSIONEND");
        wait_for(&host, "the off sound", |calls| {
            played(calls, &folder, "radio-sounds/off")
        });
        wait_until("needcfg after the game ended", || client.needcfg());
        assert!(released.elapsed() < Duration::from_millis(1_500));
        std::thread::sleep(Duration::from_millis(2_500).saturating_sub(released.elapsed()));
        assert!(
            !sent_command(&host.calls(), "TANGENT\tRELEASED"),
            "the game end cancelled the waiting release"
        );
        host.shutdown();
        assert!(host.leak_report().is_clean());
        let _ = std::fs::remove_dir_all(folder);
    }

    pub(crate) fn radio_towers_and_debug_window() {
        let (mut host, folder) = load_plugin("towers");
        host.connect_server(&ServerSetup::default()).unwrap();
        let client = game_client();
        send(
            &client,
            "RadioTwrAdd\t2:51;30000;[100,200,50]\n2:52;15000.5;[1,2,3]",
        );
        send(&client, "RadioTwrDel\t2:51");
        let diag = |host: &MockHost| {
            host.take_calls();
            host.process_command("diag").unwrap();
            host.calls().into_iter().find_map(|call| match call {
                HostCall::PrintMessageToCurrentTab { text } => Some(text),
                _ => None,
            })
        };
        wait_until("one tower left", || {
            diag(&host).is_some_and(|text| text.contains("radio towers: 1"))
        });
        host.take_calls();
        host.process_command("debug").unwrap();
        let printed: Vec<String> = host
            .calls()
            .into_iter()
            .filter_map(|call| match call {
                HostCall::PrintMessageToCurrentTab { text } => Some(text),
                _ => None,
            })
            .collect();
        assert!(
            printed.iter().any(|text| text.contains("no X11 display")),
            "without DISPLAY the debug window says why it cannot open: {printed:?}"
        );
        host.shutdown();
        assert!(host.leak_report().is_clean());
        let _ = std::fs::remove_dir_all(folder);
    }

    /// The calls that change something in TeamSpeak: everything but queries and log lines.
    fn acting_calls(calls: Vec<HostCall>) -> Vec<HostCall> {
        calls
            .into_iter()
            .filter(|call| !matches!(call, HostCall::Query(_) | HostCall::LogMessage { .. }))
            .collect()
    }

    /// Whether the latest own metadata says the game is `playing` or not.
    fn metadata_says_playing(calls: &[HostCall], playing: bool) -> bool {
        let expected = if playing {
            "Playing: [B]Yes[/B]"
        } else {
            "Playing: [B]No[/B]"
        };
        own_text_values(calls, ClientProperty::MetaData)
            .last()
            .is_some_and(|text| text.contains(expected))
    }

    /// The `FREQ` message of the local player called `nickname`.
    fn frequencies(nickname: &str) -> String {
        format!("FREQ\t[\"30.0|7|0|tf_anprc152\"]\t[]\ttrue\t20\t{nickname}\t0\t7\t1\t1\t20")
    }

    /// Run with a 10 ms service tick, so 1 s of wall time is 10 s of service ticks.
    pub(crate) fn dormant_again_after_game_end() {
        let (mut host, folder) = load_plugin("dormant-again");
        let (own, bravo, charlie) = connect_three_clients(&host);
        let client = game_client();
        client.send_focus(true).unwrap();
        start_game_in_serious_channel(&host, &client, own, &folder);
        position_three_clients(&host, &client, charlie);
        // The connect transition opens the 500 ms debounce window before the disconnect may follow.
        std::thread::sleep(Duration::from_millis(600));
        send(&client, "MISSIONEND");
        wait_for(&host, "the game end handling", |calls| {
            played(calls, &folder, "radio-sounds/off")
                && own_text_values(calls, ClientProperty::Nickname).last()
                    == Some(&"Alpha TS".to_owned())
                && metadata_says_playing(calls, false)
        });
        std::thread::sleep(Duration::from_millis(100));
        host.take_calls();

        // Ten seconds of service ticks, plugin commands, focus frames and voice: nothing acts.
        let ticks_started = Instant::now();
        host.deliver_plugin_command(bravo, REMOTE_PLUGIN_NAME, &format!("REQVOL\t{}", bravo.0))
            .unwrap();
        host.deliver_plugin_command(bravo, REMOTE_PLUGIN_NAME, "VOLUME\tBravo\t20\ttrue")
            .unwrap();
        host.deliver_plugin_command(
            bravo,
            REMOTE_PLUGIN_NAME,
            "TANGENT\tPRESSED\t30.0\t20000\tdigital\ttf_anprc152_7\t1\t0\tBravo",
        )
        .unwrap();
        client.send_focus(false).unwrap();
        client.send_focus(true).unwrap();
        let samples = sine(1_000.0, 8_000.0, 0, 480);
        assert_eq!(
            host.post_process_voice(bravo, &samples, 1).unwrap().samples,
            samples,
            "a dormant plugin passes voice through"
        );
        std::thread::sleep(Duration::from_secs(1).saturating_sub(ticks_started.elapsed()));
        let acting = acting_calls(host.calls());
        assert!(
            acting.is_empty(),
            "a plugin dormant after its game end acts on nothing: {acting:#?}"
        );

        // The next game message wakes it as the first one did.
        send(&client, &frequencies("Alpha"));
        wait_for(&host, "the game to start again", |calls| {
            played(calls, &folder, "radio-sounds/on")
                && own_text_values(calls, ClientProperty::Nickname) == ["Alpha"]
                && metadata_says_playing(calls, true)
        });
        host.shutdown();
        assert!(host.leak_report().is_clean());
        let _ = std::fs::remove_dir_all(folder);
    }

    /// The names of this process's threads the plugin starts (all begin with `tfar-`; the game
    /// link client of the test names its threads `tfar_…`), as Linux shortens them to 15 bytes.
    fn plugin_threads() -> Vec<String> {
        std::fs::read_dir("/proc/self/task")
            .unwrap()
            .filter_map(|task| std::fs::read_to_string(task.ok()?.path().join("comm")).ok())
            .map(|name| name.trim_end().to_owned())
            .filter(|name| name.starts_with("tfar-"))
            .collect()
    }

    /// How long [`slow_proxy`] holds each request before it refuses it.
    pub(crate) const SLOW_PROXY_DELAY: Duration = Duration::from_millis(500);

    /// A local HTTP proxy that answers every request with `502` after [`SLOW_PROXY_DELAY`], so a
    /// web request is still in flight at shutdown and ends soon after without network; returns
    /// its URL. It serves until the test process exits.
    pub(crate) fn slow_proxy() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for connection in listener.incoming() {
                let Ok(mut connection) = connection else {
                    continue;
                };
                std::thread::spawn(move || {
                    let mut request = [0_u8; 4_096];
                    let _ = connection.read(&mut request);
                    std::thread::sleep(SLOW_PROXY_DELAY);
                    let _ = connection.write_all(
                        b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    );
                });
            }
        });
        url
    }

    /// Run with the update check on and every proxy variable pointing at [`slow_proxy`], so the
    /// web requests are in flight when shutdown begins.
    pub(crate) fn shutdown_joins_every_thread() {
        let (mut host, folder) = load_plugin("shutdown");
        let (own, _, _) = connect_three_clients(&host);
        let client = game_client();
        client.send_focus(true).unwrap();
        start_game_in_serious_channel(&host, &client, own, &folder);
        send(&client, &frequencies("Alpha"));
        send(
            &client,
            "TRACK\tmissionStart\tsomeuid\t[[1,\"mission\",\"shutdown test\"]]",
        );
        send(
            &client,
            "TANGENT\tPRESSED\t30.0\t20000\tdigital\ttf_anprc152_1",
        );
        wait_for(&host, "the press broadcast", |calls| {
            sent_command(calls, "TANGENT\tPRESSED")
        });
        send(
            &client,
            "TANGENT\tRELEASED\t30.0\t20000\tdigital\ttf_anprc152_1",
        );
        wait_for(&host, "the release broadcast", |calls| {
            sent_command(calls, "TANGENT\tRELEASED")
        });
        let running = plugin_threads();
        for started in [
            "tfar-service",
            "tfar-game-link-",
            "tfar-tangent-re",
            "tfar-teamspeak-",
            "tfar-update-che",
            "tfar-piwik-trac",
        ] {
            assert!(
                running.iter().any(|name| name.starts_with(started)),
                "{started} runs before shutdown: {running:?}"
            );
        }

        let shutdown_started = Instant::now();
        host.shutdown();
        let shutdown_took = shutdown_started.elapsed();
        println!("shutdown took {shutdown_took:?}");
        assert!(
            shutdown_took < Duration::from_secs(2),
            "shutdown took {shutdown_took:?}"
        );
        assert_eq!(
            plugin_threads(),
            Vec::<String>::new(),
            "every thread the plugin started is joined"
        );
        assert!(host.leak_report().is_clean());
        let _ = std::fs::remove_dir_all(folder);
    }

    /// Bravo's next voice block, which the test numbers from `block`.
    fn bravo_block(host: &MockHost, bravo: ClientId, block: usize) -> Vec<i16> {
        let samples = sine(500.0, 8_000.0, block * 480, 480);
        host.post_process_voice(bravo, &samples, 1).unwrap().samples
    }

    pub(crate) fn hrtf_restart_after_an_idle_gap() {
        let (mut host, folder) = load_plugin_with_hrir("hrtf");
        let (own, bravo, charlie) = connect_three_clients(&host);
        let client = game_client();
        client.send_focus(true).unwrap();
        start_game_in_serious_channel(&host, &client, own, &folder);
        position_three_clients(&host, &client, charlie);
        assert_eq!(sync(&client, &position("Bravo", [3.0, 0.0, 0.0])), "00");
        host.deliver_plugin_command(bravo, REMOTE_PLUGIN_NAME, "VOLUME\tBravo\t20\ttrue")
            .unwrap();
        let started = Instant::now();
        let mut block = 0;
        while rms(&bravo_block(&host, bravo, block)) <= 100.0
            || rms(&bravo_block(&host, bravo, block + 1)) <= 100.0
        {
            assert!(
                started.elapsed() < EFFECT_TIMEOUT,
                "timed out waiting for Bravo's voice placed through the HRTF"
            );
            block += 2;
            std::thread::sleep(Duration::from_millis(10));
        }
        block += 2;
        host.talk_status_change(bravo, false).unwrap();
        assert!(
            rms(&bravo_block(&host, bravo, block)) > 100.0,
            "a stop leaves the voice's tail"
        );
        host.talk_status_change(bravo, true).unwrap();
        assert!(
            rms(&bravo_block(&host, bravo, block + 1)) > 100.0,
            "a talk start never cuts into the running voice"
        );
        std::thread::sleep(tfar_hrtf::IDLE_RESET_AFTER + Duration::from_millis(50));
        let first = bravo_block(&host, bravo, block + 2);
        assert!(
            rms(&first) < 1.0,
            "after the idle gap the source starts afresh, whose first block is silence: {}",
            rms(&first)
        );
        assert!(rms(&bravo_block(&host, bravo, block + 3)) > 100.0);
        host.shutdown();
        assert!(host.leak_report().is_clean());
        let _ = std::fs::remove_dir_all(folder);
    }
}

use scenarios::{
    dormant_again_after_game_end, dormant_plugin, full_scenario, hrtf_restart_after_an_idle_gap,
    mission_end_and_needcfg, radio_towers_and_debug_window, run_in_child, run_in_child_with,
    shutdown_joins_every_thread, slow_proxy,
};

#[test]
fn after_its_game_end_the_plugin_is_dormant_until_game_activity_resumes() {
    run_in_child_with(
        "after_its_game_end_the_plugin_is_dormant_until_game_activity_resumes",
        dormant_again_after_game_end,
        &[("TFAR_SERVICE_TICK_MS", "10".to_owned())],
    );
}

#[test]
fn shutdown_joins_every_thread_the_plugin_started_within_two_seconds() {
    let proxy = slow_proxy();
    let mut environment = vec![
        ("TFAR_SKIP_UPDATE_CHECK", "0".to_owned()),
        ("NO_PROXY", String::new()),
        ("no_proxy", String::new()),
    ];
    for variable in [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
    ] {
        environment.push((variable, proxy.clone()));
    }
    run_in_child_with(
        "shutdown_joins_every_thread_the_plugin_started_within_two_seconds",
        shutdown_joins_every_thread,
        &environment,
    );
}

#[test]
fn a_sender_silent_past_the_idle_gap_gets_a_fresh_hrtf_source() {
    run_in_child(
        "a_sender_silent_past_the_idle_gap_gets_a_fresh_hrtf_source",
        hrtf_restart_after_an_idle_gap,
    );
}

#[test]
fn the_full_game_scenario_mutes_renames_transmits_and_processes_radio_voice() {
    run_in_child(
        "the_full_game_scenario_mutes_renames_transmits_and_processes_radio_voice",
        full_scenario,
    );
}

#[test]
fn a_dormant_plugin_makes_no_teamspeak_side_effect_and_passes_audio_through() {
    run_in_child(
        "a_dormant_plugin_makes_no_teamspeak_side_effect_and_passes_audio_through",
        dormant_plugin,
    );
}

#[test]
fn missionend_disconnects_with_the_off_sound_and_needcfg_follows_setcfg() {
    run_in_child(
        "missionend_disconnects_with_the_off_sound_and_needcfg_follows_setcfg",
        mission_end_and_needcfg,
    );
}

#[test]
fn radio_towers_are_added_and_removed_and_the_debug_window_needs_a_display() {
    run_in_child(
        "radio_towers_are_added_and_removed_and_the_debug_window_needs_a_display",
        radio_towers_and_debug_window,
    );
}
