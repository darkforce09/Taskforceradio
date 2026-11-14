//! The built 1.0 plugin `.so` in the mock TeamSpeak host, driven by a real game link client:
//! the full game scenario, a server connected after the game started, the dormant plugin, the
//! return to dormant after a game end, the mission end and the tangent release it cancels,
//! needcfg, radio towers, the debug window without a display, the HRTF restart after an idle gap
//! (never on a talk start), and a shutdown that joins every thread within 2 s.
//!
//! The plugin reads its port and switches from the process environment, so each test re-runs its
//! own binary as a child with `TFAR_GAME_LINK_PORT` set to a free port, the update check off and
//! no X11 display (a scenario may set more variables, such as a shorter service tick or a local
//! proxy); the child loads the plugin and runs one scenario. The harness is in `support/`.

// `cfg(test)` puts the helpers under the test lint relief of the root `clippy.toml`.
#[cfg(test)]
mod support;

#[cfg(test)]
mod scenarios {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::Path;
    use std::time::{Duration, Instant};

    use tfar_game_link::client::prelude::GameLinkClient;
    use tfar_ts3_host::{ChannelId, ClientId, ClientProperty};
    use tfar_ts3_mock::{HostCall, HostQuery, MockHost, ServerSetup};

    use crate::support::{
        EFFECT_TIMEOUT, REMOTE_PLUGIN_NAME, connect_three_clients, game_client, load_plugin,
        load_plugin_with_hrir, own_int_values, own_text_values, played, position, rms, send,
        sent_command, sine, sync, wait_for, wait_until,
    };

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

    /// The game starts before TeamSpeak connects to a server: without a server the plugin plays no
    /// sound and asks TeamSpeak nothing about channels, the radio tower the game sent is kept, the
    /// server joins the running game, and a player positioned within voice range is alive and
    /// unmuted.
    pub(crate) fn server_after_game_start() {
        let (host, folder) = load_plugin("late_server");
        let client = game_client();
        client.send_focus(true).unwrap();
        wait_for(&host, "the game start without a server", |calls| {
            calls.iter().any(|call| {
                matches!(call, HostCall::LogMessage { text, .. }
                    if text.contains("the game connected is_game_start=true"))
            })
        });
        send(&client, "RadioTwrAdd\t2:51;30000;[100,200,50]");
        send(
            &client,
            "SETCFG\tserious_channelName\tTaskForceRadio\tSTRING",
        );
        send(&client, "SETCFG\tserious_channelPassword\t123\tSTRING");
        wait_until("needcfg cleared", || !client.needcfg());
        let calls = host.calls();
        assert!(
            !calls
                .iter()
                .any(|call| matches!(call, HostCall::PlayWaveFile { .. })),
            "no sound is played without a server: {calls:#?}"
        );
        assert!(
            !calls
                .iter()
                .any(|call| matches!(call, HostCall::Query(HostQuery::ChannelList(_)))),
            "no channel list is read without a server: {calls:#?}"
        );

        let (own, bravo, _) = connect_three_clients(&host);
        wait_for(&host, "the move into the serious channel", |calls| {
            calls.iter().any(|call| {
                matches!(call, HostCall::RequestClientMove { client, channel, .. }
            if *client == own && *channel == ChannelId(2))
            })
        });
        host.client_moves(own, ChannelId(2)).unwrap();
        wait_for(&host, "Bravo muted before his first position", |calls| {
            is_muted(calls, bravo)
        });

        take_radios_and_own_position(&host, &client);
        for update in [
            position("Bravo", [5.0, 0.0, 0.0]),
            position("Charlie", [900.0, 0.0, 0.0]),
        ] {
            assert_eq!(sync(&client, &update), "00");
        }
        wait_for(&host, "Bravo unmuted within voice range", |calls| {
            !is_muted(calls, bravo)
        });
        host.take_calls();
        host.process_command("diag").unwrap();
        let diag = host.calls().into_iter().find_map(|call| match call {
            HostCall::PrintMessageToCurrentTab { text } => Some(text),
            _ => None,
        });
        assert!(
            diag.as_deref()
                .is_some_and(|text| text.contains("radio towers: 1")),
            "the tower sent before the server connected is kept: {diag:?}"
        );
        let _ = std::fs::remove_dir_all(folder);
    }

    /// Whether the latest mute request naming `target` in `calls` mutes it.
    fn is_muted(calls: &[HostCall], target: ClientId) -> bool {
        calls.iter().fold(false, |muted, call| match call {
            HostCall::RequestMuteClients { clients, .. } if clients.contains(&target) => true,
            HostCall::RequestUnmuteClients { clients, .. } if clients.contains(&target) => false,
            _ => muted,
        })
    }

    /// The radios, the rename to the game name and the own position taken as the local player's.
    fn take_radios_and_own_position(host: &MockHost, client: &GameLinkClient) {
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
    }

    /// The radios, the rename, the positions and the serious-mode mute.
    fn position_three_clients(host: &MockHost, client: &GameLinkClient, charlie: ClientId) {
        take_radios_and_own_position(host, client);
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
        std::thread::sleep(tfar_audio::hrtf::IDLE_RESET_AFTER + Duration::from_millis(50));
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
    mission_end_and_needcfg, radio_towers_and_debug_window, server_after_game_start,
    shutdown_joins_every_thread, slow_proxy,
};
use support::{run_in_child, run_in_child_with};

#[test]
fn a_server_connected_after_the_game_started_hears_a_positioned_player_alive() {
    run_in_child(
        "a_server_connected_after_the_game_started_hears_a_positioned_player_alive",
        server_after_game_start,
    );
}

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
