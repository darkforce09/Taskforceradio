//! What the mock-host scenarios share: running a scenario in a child process with its own game
//! link port, a plugin folder holding the real radio sounds and the HRIR sphere, the call-log
//! filters and the test signals.

#![allow(
    dead_code,
    reason = "each scenario file uses the part of the support it needs"
)]

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use tfar_game_link_client::prelude::{ClientConfig, Flavour, GameLinkClient, SyncOutcome};
use tfar_ts3_host::{ClientProperty, ServerConnectionHandlerId};
use tfar_ts3_mock::{HostCall, HostOptions, MockHost, ServerSetup, locate_cdylib};

/// Set in the child's environment to the name of the scenario it runs.
pub(crate) const CHILD_MARKER: &str = "TFAR_PLUGIN_0_9_SCENARIO_CHILD";

/// The plugin name a 0.9.12 plugin on Linux sends its commands under.
pub(crate) const TFAR_PLUGIN_NAME: &str = "task_force_radio_linux_amd64";

/// Metadata that says a client's TFAR plugin is connected and playing.
pub(crate) const PLAYING_METADATA: &str = "<TFAR>[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y[/B] [I]P:[/I][B]0.9.12[/B][I] A: [/I][B]0.9.12[/B]</TFAR>";

/// The repository root, three folders above this crate.
pub(crate) fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// What a scenario child's environment adds to the defaults.
#[derive(Clone, Debug, Default)]
pub(crate) struct ChildEnvironment {
    /// Runs the plugin's update check instead of skipping it.
    pub(crate) checks_for_updates: bool,
    /// The `127.0.0.1` port every web request of the child goes through as its HTTP proxy
    /// (`ALL_PROXY`), so none leaves the machine.
    pub(crate) proxy_port: Option<u16>,
}

/// The proxy variables the child's web requests read; all are cleared before the child starts.
const PROXY_VARIABLES: [&str; 8] = [
    "ALL_PROXY",
    "all_proxy",
    "HTTPS_PROXY",
    "https_proxy",
    "HTTP_PROXY",
    "http_proxy",
    "NO_PROXY",
    "no_proxy",
];

/// Runs `scenario` in a child process of this test binary when this process is not already
/// that child: the child gets a free game link port, no update check and its own plugin folder.
/// In the child itself, `scenario` runs here.
pub(crate) fn run_in_child(test_name: &str, scenario: impl FnOnce()) {
    run_in_child_with(test_name, &ChildEnvironment::default(), scenario);
}

/// [`run_in_child`] with the child's environment changed as `environment` says.
pub(crate) fn run_in_child_with(
    test_name: &str,
    environment: &ChildEnvironment,
    scenario: impl FnOnce(),
) {
    if std::env::var(CHILD_MARKER).as_deref() == Ok(test_name) {
        scenario();
        return;
    }
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([test_name, "--exact", "--nocapture", "--test-threads=1"])
        .env(CHILD_MARKER, test_name)
        .env("TFAR_GAME_LINK_PORT", free_port().to_string());
    if environment.checks_for_updates {
        command.env_remove("TFAR_SKIP_UPDATE_CHECK");
    } else {
        command.env("TFAR_SKIP_UPDATE_CHECK", "1");
    }
    for variable in PROXY_VARIABLES {
        command.env_remove(variable);
    }
    if let Some(port) = environment.proxy_port {
        command.env("ALL_PROXY", format!("http://127.0.0.1:{port}"));
    }
    let status = command.status().unwrap();
    assert!(
        status.success(),
        "the scenario failed in the child: {status}"
    );
}

/// The names of this process's threads that the plugin or its libraries started: every name
/// starts with `tfar-` (Linux keeps the first 15 bytes).
pub(crate) fn plugin_thread_names() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir("/proc/self/task")
        .unwrap()
        .filter_map(|task| std::fs::read_to_string(task.ok()?.path().join("comm")).ok())
        .map(|name| name.trim_end().to_owned())
        .filter(|name| name.starts_with("tfar-"))
        .collect();
    names.sort_unstable();
    names
}

/// Waits up to `timeout` for `holds` to be true of the host's call log.
pub(crate) fn wait_for_calls_within(
    host: &MockHost,
    timeout: Duration,
    holds: impl Fn(&[HostCall]) -> bool,
) -> Vec<HostCall> {
    let deadline = Instant::now() + timeout;
    loop {
        let calls = host.calls();
        if holds(&calls) || Instant::now() > deadline {
            return calls;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// A port nothing listens on right now: bound once, then released.
pub(crate) fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// The game link port the child's plugin listens on.
pub(crate) fn child_port() -> u16 {
    std::env::var("TFAR_GAME_LINK_PORT")
        .unwrap()
        .parse()
        .unwrap()
}

/// A fresh plugin folder for this process, holding the 0.9.12 `radio-sounds/` from `reference/`
/// and `tfar_hrir/IRC_1002_C.bin` from `assets/hrir/`, as `cargo xtask dist` lays them out.
pub(crate) fn plugin_folder() -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("tfar_plugin_0_9_scenario_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    let root = repository_root();
    copy_folder(
        &root.join("reference/tfar_0_9_12/ts/radio-sounds"),
        &folder.join("radio-sounds"),
    );
    let hrir = folder.join("tfar_hrir");
    std::fs::create_dir_all(&hrir).unwrap();
    std::fs::copy(
        root.join("assets/hrir/IRC_1002_C.bin"),
        hrir.join("IRC_1002_C.bin"),
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

/// Loads the built plugin with `plugin_folder` as TeamSpeak's plugins folder.
pub(crate) fn load_plugin(plugin_folder: &Path) -> MockHost {
    let library = locate_cdylib("tfar_plugin_0_9").expect("libtfar_plugin_0_9.so is built");
    MockHost::load_with_options(
        &library,
        &HostOptions {
            plugin_path: Some(plugin_folder.to_owned()),
            config_path: None,
        },
    )
    .unwrap()
}

/// Connects a server with the channels `Lobby` and `Serious`, us as `Mock Player` in the lobby.
pub(crate) fn connect(host: &MockHost) -> ServerConnectionHandlerId {
    host.connect_server(&ServerSetup {
        own_nickname: "Mock Player".to_owned(),
        channel_names: vec!["Lobby".to_owned(), "Serious".to_owned()],
        ..ServerSetup::default()
    })
    .unwrap()
}

/// A game link client of the 0.9 flavour on the child's port.
pub(crate) fn game_link() -> GameLinkClient {
    GameLinkClient::new(ClientConfig::new(Flavour::V0_9, child_port()))
}

/// Sends `message` and returns the reply text.
pub(crate) fn send(client: &GameLinkClient, message: &str) -> String {
    match client.sync_request(message.as_bytes()) {
        SyncOutcome::Reply(reply) => String::from_utf8(reply).unwrap(),
        other => panic!("{message:?} got no reply: {other:?}"),
    }
}

/// Waits up to two seconds for `holds` to be true of the host's call log.
pub(crate) fn wait_for_calls(
    host: &MockHost,
    holds: impl Fn(&[HostCall]) -> bool,
) -> Vec<HostCall> {
    wait_for_calls_within(host, Duration::from_secs(2), holds)
}

/// The plugin commands in `calls`.
pub(crate) fn plugin_commands(calls: &[HostCall]) -> Vec<String> {
    calls
        .iter()
        .filter_map(|call| match call {
            HostCall::SendPluginCommand { command, .. } => Some(command.clone()),
            _ => None,
        })
        .collect()
}

/// The own string variables `calls` set for `property`, in order.
pub(crate) fn own_strings_set(calls: &[HostCall], property: ClientProperty) -> Vec<String> {
    calls
        .iter()
        .filter_map(|call| match call {
            HostCall::SetClientSelfVariableAsString { flag, value, .. }
                if *flag == property.flag() =>
            {
                Some(value.clone())
            }
            _ => None,
        })
        .collect()
}

/// Whether `calls` hold anything a dormant plugin must not do.
pub(crate) fn has_dormant_forbidden_call(calls: &[HostCall]) -> bool {
    calls.iter().any(|call| {
        matches!(
            call,
            HostCall::SendPluginCommand { .. }
                | HostCall::SetClientSelfVariableAsString { .. }
                | HostCall::RequestMuteClients { .. }
                | HostCall::RequestUnmuteClients { .. }
                | HostCall::RequestClientMove { .. }
        )
    })
}

#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    reason = "a test signal: frame indices stay far below 2^52 and the amplitude within i16"
)]
/// `frames` frames of a stereo sine at `frequency_hz` and `amplitude`, starting at frame
/// `first_frame`.
pub(crate) fn stereo_tone(
    frequency_hz: f64,
    amplitude: f64,
    first_frame: usize,
    frames: usize,
) -> Vec<i16> {
    (first_frame..first_frame + frames)
        .flat_map(|frame| {
            let phase = std::f64::consts::TAU * frequency_hz * frame as f64 / 48_000.0;
            let sample = (amplitude * phase.sin()) as i16;
            [sample, sample]
        })
        .collect()
}

/// The root mean square of `samples`.
#[expect(
    clippy::cast_precision_loss,
    reason = "a test measure: sample counts stay far below 2^52"
)]
pub(crate) fn rms(samples: &[i16]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let energy: f64 = samples
        .iter()
        .map(|sample| f64::from(*sample).powi(2))
        .sum();
    (energy / samples.len() as f64).sqrt()
}
