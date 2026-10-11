//! The built plugin `.so` in the mock TeamSpeak host, driven like a game session: a server with
//! three TFAR players, game messages over a real game link client, a remote push-to-talk, voice
//! through the post-process callback, the remote sound in the mixed playback, and a clean
//! shutdown; and the same host with no game activity, where the plugin stays dormant.
//!
//! Each scenario runs in a child process of this test binary (`scenario_support::run_in_child`),
//! with its own game link port and plugin folder.

#[cfg(test)]
mod scenario_support;

#[cfg(test)]
mod scenarios {
    use std::time::Duration;

    use tfar_game_link::client::prelude::GameLinkClient;
    use tfar_ts3_host::{ChannelId, ClientId, ClientProperty, ServerConnectionHandlerId};
    use tfar_ts3_mock::{HostCall, MockHost};

    use super::scenario_support::{
        PLAYING_METADATA, TFAR_PLUGIN_NAME, connect, game_link, has_dormant_forbidden_call,
        load_plugin, own_strings_set, plugin_commands, plugin_folder, rms, run_in_child, send,
        stereo_tone, wait_for_calls,
    };

    const LOBBY: ChannelId = ChannelId(1);
    const SERIOUS: ChannelId = ChannelId(2);
    const ME: ClientId = ClientId(1);

    /// A voice block of 20 ms.
    const BLOCK_FRAMES: usize = 960;

    /// The local player's radios: one short-range radio on `100_alpha` at volume 7, alive, and
    /// the game's nickname `Sgt Mock`.
    const FREQ: &str =
        "FREQ\t[\"100_alpha|7|0\"]\t[]\t50\ttrue\t20\t7\tSgt Mock\t0\t7\t1\t20\t1\t20";

    /// The status the plugin publishes in game with the link up.
    const IN_GAME_METADATA: &str = "<TFAR>[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y[/B] [I]P:[/I][B]0.9.12[/B][I] A: [/I][B]0.9.12[/B]</TFAR>";

    /// The players of the scenario.
    struct Players {
        alpha: ClientId,
        bravo: ClientId,
        charlie: ClientId,
        delta: ClientId,
    }

    /// Sets up the server: three TFAR players and one without TFAR, the TFAR players' metadata
    /// published.
    fn populate(host: &MockHost) -> Players {
        let players = Players {
            alpha: host.client_joins("Alpha", LOBBY).unwrap(),
            bravo: host.client_joins("Bravo", LOBBY).unwrap(),
            charlie: host.client_joins("Charlie", LOBBY).unwrap(),
            delta: host.client_joins("Delta", LOBBY).unwrap(),
        };
        for client in [players.alpha, players.bravo, players.charlie] {
            host.set_client_metadata(client, PLAYING_METADATA).unwrap();
        }
        players
    }

    /// Pushes `blocks` blocks of a `frequency_hz` tone from `client` through the post-process
    /// callback; returns the RMS of the second half of the output, after the radio effect's
    /// delay line has filled, and whether any block came out changed.
    fn voice_rms(
        host: &MockHost,
        client: ClientId,
        frequency_hz: f64,
        blocks: usize,
    ) -> (f64, bool) {
        let mut tail = Vec::new();
        let mut is_changed = false;
        for block in 0..blocks {
            let input = stereo_tone(frequency_hz, 8_000.0, block * BLOCK_FRAMES, BLOCK_FRAMES);
            let output = host.post_process_voice(client, &input, 2).unwrap().samples;
            is_changed |= output != input;
            if block >= blocks / 2 {
                tail.extend(output);
            }
        }
        (rms(&tail), is_changed)
    }

    /// Whether a post-process block from `client` comes out exactly as it went in.
    fn passes_voice_through(host: &MockHost, client: ClientId) -> bool {
        let voice = stereo_tone(1_000.0, 8_000.0, 0, BLOCK_FRAMES);
        host.post_process_voice(client, &voice, 2).unwrap().samples == voice
    }

    #[test]
    fn a_game_session_drives_teamspeak_and_the_voice_chain() {
        run_in_child(
            "scenarios::a_game_session_drives_teamspeak_and_the_voice_chain",
            game_session_scenario,
        );
    }

    fn game_session_scenario() {
        let folder = plugin_folder();
        let mut host = load_plugin(&folder);
        let server = connect(&host);
        let players = populate(&host);
        let before_game = host.take_calls();
        assert!(!has_dormant_forbidden_call(&before_game), "{before_game:?}");
        assert!(before_game.contains(&HostCall::Systemset3DSettings {
            server,
            distance_factor: 1.0,
            rolloff_scale: 1.0,
        }));
        let link = game_link();
        start_the_game(&host, &link, server);
        position_everyone(&host, &link, &players);
        press_and_release(&host, &link, server);
        hear_a_remote_transmission(&host, &link, server, &players);
        assert_eq!(send(&link, "TS_INFO\tPING"), "PONG");
        host.take_calls();
        let stopping = std::time::Instant::now();
        host.shutdown();
        assert!(stopping.elapsed() < Duration::from_secs(2));
        let leaks = host.leak_report();
        assert!(
            leaks.is_clean(),
            "the plugin leaked TeamSpeak memory: {leaks:?}"
        );
        let shutdown = host.calls();
        assert_eq!(
            own_strings_set(&shutdown, ClientProperty::Nickname),
            vec!["Mock Player"]
        );
        assert_eq!(
            own_strings_set(&shutdown, ClientProperty::MetaData)
                .last()
                .map(String::as_str),
            Some("<TFAR>[B]Task Force Radio Plugin Disabled[/B]</TFAR>")
        );
        let _ = std::fs::remove_dir_all(folder);
    }

    /// The ping starts the game before the handshake, as Arma sends them; the handshake then
    /// still moves into the serious channel, and `FREQ` renames to the game's nickname.
    fn start_the_game(host: &MockHost, link: &GameLinkClient, server: ServerConnectionHandlerId) {
        assert_eq!(send(link, "TS_INFO\tPING"), "PONG");
        assert_eq!(send(link, "VERSION\t0.9.12\tSerious\tpw"), "OK");
        assert_eq!(send(link, FREQ), "OK");
        let calls = host.take_calls();
        assert_eq!(
            own_strings_set(&calls, ClientProperty::Nickname),
            vec!["Sgt Mock"]
        );
        assert!(calls.contains(&HostCall::RequestClientMove {
            server,
            client: ME,
            channel: SERIOUS,
            password: "pw".to_owned(),
        }));
        assert!(calls.iter().any(|call| matches!(
            call,
            HostCall::PlayWaveFile { path, .. } if path.ends_with("radio-sounds/on.wav")
        )));
        assert_eq!(
            own_strings_set(&calls, ClientProperty::MetaData)
                .last()
                .map(String::as_str),
            Some(IN_GAME_METADATA)
        );
    }

    /// Everyone moves into the serious channel; the silent players out of voice range are
    /// muted.
    fn position_everyone(host: &MockHost, link: &GameLinkClient, players: &Players) {
        for client in [ME, players.alpha, players.bravo, players.charlie] {
            host.client_moves(client, SERIOUS).unwrap();
        }
        for message in [
            "POS\tSgt Mock\t0\t0\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0",
            "POS\tAlpha\t3\t4\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0",
            "POS\tBravo\t40\t0\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0",
            "POS\tCharlie\t0\t60\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0",
        ] {
            assert_eq!(send(link, message), "NOT_SPEAKING");
        }
        let mut muted: Vec<ClientId> = host
            .take_calls()
            .iter()
            .filter_map(|call| match call {
                HostCall::RequestMuteClients { clients, .. } => Some(clients.clone()),
                _ => None,
            })
            .flatten()
            .collect();
        muted.sort_unstable();
        assert_eq!(muted, vec![players.alpha, players.bravo, players.charlie]);
    }

    /// The local push-to-talk: the microphone on and the press broadcast, then after the
    /// release the microphone off and the release broadcast.
    fn press_and_release(
        host: &MockHost,
        link: &GameLinkClient,
        server: ServerConnectionHandlerId,
    ) {
        let input_flag = ClientProperty::InputDeactivated.flag();
        assert_eq!(
            send(link, "TANGENT\tPRESSED\t100_alpha\t5000\tdigital"),
            "OK"
        );
        let pressed = host.take_calls();
        assert!(pressed.contains(&HostCall::SetClientSelfVariableAsInt {
            server,
            flag: input_flag,
            value: 0,
        }));
        assert_eq!(
            plugin_commands(&pressed),
            vec!["TANGENT\tPRESSED\t100_alpha\t5000\tdigital\tSgt Mock"]
        );
        assert_eq!(
            send(link, "TANGENT\tRELEASED\t100_alpha\t5000\tdigital"),
            "OK"
        );
        let released = wait_for_calls(host, |calls| plugin_commands(calls).len() == 1);
        assert!(released.contains(&HostCall::SetClientSelfVariableAsInt {
            server,
            flag: input_flag,
            value: 1,
        }));
        assert_eq!(
            plugin_commands(&released),
            vec!["TANGENT\tRELEASED\t100_alpha\t5000\tdigital\tSgt Mock"]
        );
        host.take_calls();
    }

    /// Alpha keys the radio we listen on: Alpha is unmuted, the remote start sound plays in the
    /// mixed playback, Alpha's voice comes out band-limited by the radio effect, and Delta, who
    /// runs no TFAR, passes through untouched.
    fn hear_a_remote_transmission(
        host: &MockHost,
        link: &GameLinkClient,
        server: ServerConnectionHandlerId,
        players: &Players,
    ) {
        host.deliver_plugin_command(
            players.alpha,
            TFAR_PLUGIN_NAME,
            "TANGENT\tPRESSED\t100_alpha\t5000\tdigital\tAlpha",
        )
        .unwrap();
        assert!(host.take_calls().contains(&HostCall::RequestUnmuteClients {
            server,
            clients: vec![players.alpha],
        }));
        let mixed = host.mixed_playback(&[0; BLOCK_FRAMES * 2], 2, 0).unwrap();
        assert!(mixed.samples.iter().any(|sample| *sample != 0));
        assert_eq!(mixed.channel_fill_mask & 3, 3);
        assert_eq!(send(link, "TS_INFO\tPING"), "PONG");
        let (in_band, in_band_changed) = voice_rms(host, players.alpha, 1_500.0, 20);
        let (high, _) = voice_rms(host, players.alpha, 12_000.0, 20);
        eprintln!("radio voice RMS: 1.5 kHz {in_band:.1}, 12 kHz {high:.1}");
        assert!(in_band_changed);
        assert!(in_band > 100.0, "the radio voice is audible: {in_band}");
        assert!(
            high < in_band * 0.25,
            "12 kHz {high} against 1.5 kHz {in_band}"
        );
        assert!(passes_voice_through(host, players.delta));
    }

    #[test]
    fn without_game_activity_the_plugin_stays_dormant() {
        run_in_child(
            "scenarios::without_game_activity_the_plugin_stays_dormant",
            dormant_scenario,
        );
    }

    fn dormant_scenario() {
        let folder = plugin_folder();
        let mut host = load_plugin(&folder);
        connect(&host);
        let players = populate(&host);
        host.deliver_plugin_command(players.alpha, TFAR_PLUGIN_NAME, "VOLUME\tAlpha\t40\ttrue")
            .unwrap();
        host.deliver_plugin_command(
            players.alpha,
            TFAR_PLUGIN_NAME,
            "TANGENT\tPRESSED\t100_alpha\t5000\tdigital\tAlpha",
        )
        .unwrap();
        let link = game_link();
        assert_eq!(send(&link, "VERSION\t0.9.12\tSerious\tpw"), "OK");
        assert!(passes_voice_through(&host, players.alpha));
        let voice = stereo_tone(1_000.0, 8_000.0, 0, BLOCK_FRAMES);
        let mixed = host.mixed_playback(&voice, 2, 0).unwrap();
        let is_mix_untouched = mixed.samples == voice && mixed.channel_fill_mask == 0;
        assert!(is_mix_untouched);
        let is_capture_untouched = host.captured_voice(&voice, 2, 2).unwrap().samples == voice;
        assert!(is_capture_untouched);
        std::thread::sleep(Duration::from_millis(300));
        host.shutdown();
        let calls = host.calls();
        assert!(!has_dormant_forbidden_call(&calls), "{calls:?}");
        assert!(
            !calls
                .iter()
                .any(|call| matches!(call, HostCall::PlayWaveFile { .. }))
        );
        assert!(host.leak_report().is_clean());
        let _ = std::fs::remove_dir_all(folder);
    }
}
