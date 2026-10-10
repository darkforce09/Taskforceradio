//! The built plugin `.so` in the mock TeamSpeak host around the end of a game and the end of the
//! plugin: dormant again after the game times out until game activity resumes, a sender's HRTF
//! sources restarting after an idle gap but never on a talk start, and a shutdown that joins every thread the plugin
//! started within TeamSpeak's 2 s, also when a web request hangs.
//!
//! Each scenario runs in a child process of this test binary (`scenario_support::run_in_child`),
//! with its own game link port and plugin folder; web requests go through a proxy on 127.0.0.1,
//! so none leaves the machine.

#[cfg(test)]
mod scenario_support;

#[cfg(test)]
mod scenarios {
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    use tfar_game_link_client::prelude::GameLinkClient;
    use tfar_ts3_host::{ClientId, ClientProperty, ServerConnectionHandlerId, Ts3Vector};
    use tfar_ts3_mock::{HostCall, MockHost};

    use super::scenario_support::{
        ChildEnvironment, TFAR_PLUGIN_NAME, connect, free_port, game_link,
        has_dormant_forbidden_call, load_plugin, own_strings_set, plugin_folder,
        plugin_thread_names, run_in_child, run_in_child_with, send, stereo_tone,
        wait_for_calls_within,
    };

    /// A voice block of 20 ms.
    const BLOCK_FRAMES: usize = 960;

    /// The handshake without a serious channel.
    const VERSION: &str = "VERSION\t0.9.12\t\t";

    /// The local player's radios and the game's nickname `Sgt Mock`.
    const FREQ: &str =
        "FREQ\t[\"100_alpha|7|0\"]\t[]\t50\ttrue\t20\t7\tSgt Mock\t0\t7\t1\t20\t1\t20";

    /// The local player at the origin.
    const MY_POSITION: &str = "POS\tSgt Mock\t0\t0\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0";

    /// Alpha five metres away.
    const ALPHA_POSITION: &str = "POS\tAlpha\t3\t4\t0\t0\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0";

    /// The metadata Alpha's plugin publishes.
    const PLAYING_METADATA: &str = "<TFAR>[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y[/B] [I]P:[/I][B]0.9.12[/B][I] A: [/I][B]0.9.12[/B]</TFAR>";

    /// Our own client.
    const ME: ClientId = ClientId(1);

    /// A status block another radio plugin writes into our metadata.
    const OTHER_PLUGIN_METADATA: &str =
        "<TFAR>[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y[/B] [I]P:[/I][B]1.0[/B]</TFAR>";

    /// The longest `ts3plugin_shutdown` may take.
    const SHUTDOWN_BUDGET: Duration = Duration::from_secs(2);

    /// Alpha joins with TFAR metadata, the game starts with both players positioned, and
    /// Alpha's voice reaches 20 m. Returns Alpha.
    fn start_a_game(host: &MockHost, link: &GameLinkClient) -> ClientId {
        let alpha = host
            .client_joins("Alpha", tfar_ts3_host::ChannelId(1))
            .unwrap();
        host.set_client_metadata(alpha, PLAYING_METADATA).unwrap();
        for message in [VERSION, FREQ] {
            assert_eq!(send(link, message), "OK");
        }
        for message in [MY_POSITION, ALPHA_POSITION] {
            assert_eq!(send(link, message), "NOT_SPEAKING");
        }
        host.deliver_plugin_command(alpha, TFAR_PLUGIN_NAME, "VOLUME\tAlpha\t20\tfalse")
            .unwrap();
        alpha
    }

    fn plays(calls: &[HostCall], file: &str) -> bool {
        calls
            .iter()
            .any(|call| matches!(call, HostCall::PlayWaveFile { path, .. } if path.ends_with(file)))
    }

    #[test]
    fn after_the_game_ends_the_plugin_stays_dormant_until_the_game_resumes() {
        run_in_child(
            "scenarios::after_the_game_ends_the_plugin_stays_dormant_until_the_game_resumes",
            game_end_scenario,
        );
    }

    fn game_end_scenario() {
        let folder = plugin_folder();
        let mut host = load_plugin(&folder);
        let server = connect(&host);
        let link = game_link();
        let alpha = start_a_game(&host, &link);
        let started = host.take_calls();
        assert!(started.contains(&HostCall::Systemset3DListenerAttributes {
            server,
            position: Some(Ts3Vector::default()),
            forward: None,
            up: None,
        }));

        // No game message for 4 s ends the game.
        let ended = wait_for_calls_within(&host, Duration::from_secs(6), |calls| {
            plays(calls, "off.wav")
        });
        assert!(plays(&ended, "off.wav"), "{ended:?}");
        std::thread::sleep(Duration::from_millis(300));
        let ended = host.take_calls();
        assert_eq!(
            own_strings_set(&ended, ClientProperty::Nickname),
            vec!["Mock Player"]
        );
        let metadata = own_strings_set(&ended, ClientProperty::MetaData);
        assert_eq!(metadata.len(), 1, "{ended:?}");
        assert!(
            metadata
                .iter()
                .all(|text| text.contains("[I]Play[/I] [B]N[/B]"))
        );

        // Ten seconds of service ticks, remote commands and voice, with no side effect, while
        // another radio plugin (the 1.0 one, say) writes its own status into our metadata.
        host.set_client_metadata(ME, OTHER_PLUGIN_METADATA).unwrap();
        let watched = Instant::now();
        while watched.elapsed() < Duration::from_secs(10) {
            host.deliver_plugin_command(alpha, TFAR_PLUGIN_NAME, "VOLUME\tAlpha\t20\ttrue")
                .unwrap();
            host.deliver_plugin_command(
                alpha,
                TFAR_PLUGIN_NAME,
                "TANGENT\tPRESSED\t100_alpha\t5000\tdigital\tAlpha",
            )
            .unwrap();
            let voice = stereo_tone(1_000.0, 8_000.0, 0, BLOCK_FRAMES);
            assert_eq!(
                host.post_process_voice(alpha, &voice, 2).unwrap().samples,
                voice
            );
            assert_eq!(send(&link, VERSION), "OK");
            std::thread::sleep(Duration::from_millis(500));
        }
        let dormant = host.take_calls();
        assert!(!has_dormant_forbidden_call(&dormant), "{dormant:?}");
        assert!(
            !dormant
                .iter()
                .any(|call| matches!(call, HostCall::PlayWaveFile { .. })),
            "{dormant:?}"
        );

        // Game activity wakes the plugin again.
        assert_eq!(send(&link, FREQ), "OK");
        let resumed = host.take_calls();
        assert!(plays(&resumed, "on.wav"), "{resumed:?}");
        assert_eq!(
            own_strings_set(&resumed, ClientProperty::Nickname),
            vec!["Sgt Mock"]
        );
        assert!(
            own_strings_set(&resumed, ClientProperty::MetaData)
                .iter()
                .any(|text| text.contains("[I]Play[/I] [B]Y[/B]")),
            "{resumed:?}"
        );
        host.shutdown();
        assert!(host.leak_report().is_clean());
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn a_sender_silent_past_the_idle_gap_never_replays_the_old_tail() {
        run_in_child(
            "scenarios::a_sender_silent_past_the_idle_gap_never_replays_the_old_tail",
            talk_start_scenario,
        );
    }

    /// Alpha's voice, then a silent block: the HRTF's held tail of the voice comes out unless
    /// the sources restarted in between. `starts_talking` sends a talk start before the silent
    /// block, `pauses` waits past the idle gap.
    fn silent_block_after_voice(
        host: &MockHost,
        alpha: ClientId,
        starts_talking: bool,
        pauses: bool,
    ) -> bool {
        for block in 0..5 {
            let voice = stereo_tone(700.0, 8_000.0, block * BLOCK_FRAMES, BLOCK_FRAMES);
            host.post_process_voice(alpha, &voice, 2).unwrap();
        }
        if starts_talking {
            host.talk_status_change(alpha, true).unwrap();
        }
        if pauses {
            std::thread::sleep(tfar_hrtf::IDLE_RESET_AFTER + Duration::from_millis(50));
        }
        let silence = vec![0_i16; BLOCK_FRAMES * 2];
        let output = host.post_process_voice(alpha, &silence, 2).unwrap().samples;
        host.talk_status_change(alpha, false).unwrap();
        output.iter().all(|sample| *sample == 0)
    }

    fn talk_start_scenario() {
        let folder = plugin_folder();
        let mut host = load_plugin(&folder);
        connect(&host);
        let link = game_link();
        let alpha = start_a_game(&host, &link);
        assert!(
            !silent_block_after_voice(&host, alpha, false, false),
            "without a gap the tail of the voice plays on"
        );
        assert_eq!(send(&link, ALPHA_POSITION), "NOT_SPEAKING");
        assert!(
            !silent_block_after_voice(&host, alpha, true, false),
            "a talk start never cuts into the running voice"
        );
        assert_eq!(send(&link, ALPHA_POSITION), "NOT_SPEAKING");
        assert!(
            silent_block_after_voice(&host, alpha, false, true),
            "the idle gap restarted the sources"
        );
        host.take_calls();
        host.talk_status_change(alpha, true).unwrap();
        assert!(!has_dormant_forbidden_call(&host.take_calls()));
        host.shutdown();
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn shutdown_joins_every_plugin_thread_within_two_seconds() {
        run_in_child_with(
            "scenarios::shutdown_joins_every_plugin_thread_within_two_seconds",
            &ChildEnvironment {
                checks_for_updates: true,
                proxy_port: Some(free_port()),
            },
            shutdown_scenario,
        );
    }

    fn shutdown_scenario() {
        let folder = plugin_folder();
        let mut host = load_plugin(&folder);
        let server: ServerConnectionHandlerId = connect(&host);
        let link = game_link();
        start_a_game(&host, &link);
        for path in ["mission/start", "mission/radio"] {
            assert_eq!(send(&link, &format!("TRACK\t{path}")), "OK");
        }
        assert_eq!(
            send(&link, "TANGENT\tPRESSED\t100_alpha\t5000\tdigital"),
            "OK"
        );
        let running = plugin_thread_names();
        for expected in [
            "tfar-service",
            "tfar-push-to-ta",
            "tfar-teamspeak-",
            "tfar-game-link-",
        ] {
            assert!(
                running.iter().any(|name| name == expected),
                "{expected} in {running:?}"
            );
        }
        let stopping = Instant::now();
        host.shutdown();
        let took = stopping.elapsed();
        eprintln!("the shutdown took {took:?}");
        assert!(took < SHUTDOWN_BUDGET, "{took:?}");
        assert_eq!(plugin_thread_names(), Vec::<String>::new());
        assert!(host.leak_report().is_clean());
        assert!(host.calls().iter().any(|call| matches!(
            call,
            HostCall::SetClientSelfVariableAsString { server: on, .. } if *on == server
        )));
        drop(link);
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn a_web_request_in_flight_never_holds_the_shutdown_past_two_seconds() {
        // The parent's listener takes the request into its backlog and never answers.
        let silent_proxy = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = silent_proxy.local_addr().unwrap().port();
        run_in_child_with(
            "scenarios::a_web_request_in_flight_never_holds_the_shutdown_past_two_seconds",
            &ChildEnvironment {
                checks_for_updates: false,
                proxy_port: Some(port),
            },
            hanging_request_scenario,
        );
        drop(silent_proxy);
    }

    fn hanging_request_scenario() {
        let folder = plugin_folder();
        let mut host = load_plugin(&folder);
        connect(&host);
        let link = game_link();
        start_a_game(&host, &link);
        assert_eq!(send(&link, "TRACK\tmission/start"), "OK");
        std::thread::sleep(Duration::from_millis(200));
        let stopping = Instant::now();
        host.shutdown();
        let took = stopping.elapsed();
        eprintln!("the shutdown with a hanging request took {took:?}");
        assert!(took < SHUTDOWN_BUDGET, "{took:?}");
        assert!(took >= Duration::from_millis(1_400), "{took:?}");
        assert_eq!(plugin_thread_names(), vec!["tfar-piwik-trac"]);
        // The cancelled request ends within its own 5 s timeout; the library stays loaded.
        let waiting = Instant::now();
        while !plugin_thread_names().is_empty() && waiting.elapsed() < Duration::from_secs(7) {
            std::thread::sleep(Duration::from_millis(50));
        }
        assert_eq!(plugin_thread_names(), Vec::<String>::new());
        drop(link);
        drop(host);
        let _ = std::fs::remove_dir_all(folder);
    }
}
