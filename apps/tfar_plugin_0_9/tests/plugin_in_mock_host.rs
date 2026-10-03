//! The built plugin `.so` in the mock TeamSpeak host: TeamSpeak's load order, the original
//! metadata, a game link round trip to the stub handler, and a clean shutdown.
//!
//! The plugin reads its port from the process environment, so the test re-runs its own binary as
//! a child with `TFAR_GAME_LINK_PORT` set to a free port; the child loads the plugin and runs the
//! scenario. No other test shares the process, and the fixed ports stay free for TeamSpeak.

use std::process::Command;

use mock_host_scenario::{CHILD_MARKER, free_port, run_scenario};

/// The name of the one test, which the parent passes to the child's test harness.
const TEST_NAME: &str = "the_plugin_loads_answers_the_ping_and_shuts_down";

#[cfg(test)]
mod mock_host_scenario {
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    use tfar_game_link_client::prelude::{ClientConfig, Flavour, GameLinkClient, SyncOutcome};
    use tfar_ts3_mock::{MockHost, locate_cdylib};

    /// Set in the child's environment; its presence means "run the scenario here".
    pub(crate) const CHILD_MARKER: &str = "TFAR_PLUGIN_0_9_MOCK_HOST_CHILD";

    /// A port nothing listens on right now: bound once, then released.
    pub(crate) fn free_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    }

    /// Loads the plugin, checks its metadata, pings it over the game link and shuts it down.
    pub(crate) fn run_scenario() {
        let port: u16 = std::env::var("TFAR_GAME_LINK_PORT")
            .unwrap()
            .parse()
            .unwrap();
        let library = locate_cdylib("tfar_plugin_0_9").expect("libtfar_plugin_0_9.so is built");
        let mut host = MockHost::load(&library).unwrap();

        let metadata = host.metadata();
        assert_eq!(metadata.name, "Task Force Arma 3 Radio");
        assert_eq!(metadata.version, "0.9.12");
        assert_eq!(metadata.api_version, 26);
        assert_eq!(metadata.author, "[TF]Nkey");
        assert_eq!(metadata.description, "Radio Addon for Arma 3");
        assert_eq!(
            metadata.info_title.as_deref(),
            Some("Task Force Radio Status (0.9.12)")
        );
        assert_eq!(metadata.command_keyword.as_deref(), Some(""));
        assert!(metadata.requests_autoload);
        assert_eq!(metadata.offers_configure, Some(0));

        let client = GameLinkClient::new(ClientConfig::new(Flavour::V0_9, port));
        let started = Instant::now();
        assert_eq!(
            client.sync_request(b"TS_INFO\tPING"),
            SyncOutcome::Reply(b"PONG".to_vec())
        );
        assert_eq!(
            client.sync_request(b"TS_INFO\tVERSION"),
            SyncOutcome::Reply(b"FAIL".to_vec())
        );
        assert!(started.elapsed() < Duration::from_secs(1));

        host.shutdown();
        let leaks = host.leak_report();
        assert!(
            leaks.is_clean(),
            "the plugin leaked TeamSpeak memory: {leaks:?}"
        );

        // After shutdown the listener is closed, so a new client finds no plugin.
        let after = GameLinkClient::new(ClientConfig::new(Flavour::V0_9, port));
        assert_eq!(after.sync_request(b"TS_INFO\tPING"), SyncOutcome::NoSession);
    }
}

#[test]
fn the_plugin_loads_answers_the_ping_and_shuts_down() {
    if std::env::var_os(CHILD_MARKER).is_some() {
        run_scenario();
        return;
    }
    let status = Command::new(std::env::current_exe().unwrap())
        .args([TEST_NAME, "--exact", "--nocapture", "--test-threads=1"])
        .env(CHILD_MARKER, "1")
        .env("TFAR_SKIP_UPDATE_CHECK", "1")
        .env("TFAR_GAME_LINK_PORT", free_port().to_string())
        .env_remove("TFAR_GAME_LINK_DEBUG")
        .status()
        .unwrap();
    assert!(
        status.success(),
        "the scenario failed in the child: {status}"
    );
}
