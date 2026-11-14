//! The harness the mock-host game scenarios share: each scenario in a child process of its test
//! binary, the built plugin loaded with a generated plugins folder, a game link client, waits for
//! the plugin's effects, and readers of the host calls and of the audio it returns.

use std::f64::consts::TAU;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use tfar_game_link::client::prelude::{ClientConfig, Flavour, GameLinkClient, SyncOutcome};
use tfar_ts3_host::{ChannelId, ClientId, ClientProperty};
use tfar_ts3_mock::{
    HostCall, HostOptions, MockHost, SentCommandTarget, ServerSetup, locate_cdylib,
};

/// Set in a child's environment to the name of the scenario it runs.
const CHILD_MARKER: &str = "TFAR_PLUGIN_1_0_SCENARIO_CHILD";

/// How long a scenario waits for an effect of an async message.
pub(crate) const EFFECT_TIMEOUT: Duration = Duration::from_secs(3);

/// The plugin name other TFAR clients send plugin commands with.
pub(crate) const REMOTE_PLUGIN_NAME: &str = "TFAR_linux_amd64";

/// The metadata of a client that runs TFAR and plays.
const PLAYING_METADATA: &str = "<TFAR>Connected to Game: [B]Yes[/B]\nPlaying: [B]Yes[/B]\nPlugin \
                            version: [B]1.0.0.1[/B]\nAddon version: [B]1.0[/B]\n</TFAR>";

/// Runs `scenario` in a child process of this test binary, or right here inside the child.
pub(crate) fn run_in_child(test_name: &str, scenario: fn()) {
    run_in_child_with(test_name, scenario, &[]);
}

/// [`run_in_child`] with the variables of `environment` set in the child, over the defaults.
pub(crate) fn run_in_child_with(test_name: &str, scenario: fn(), environment: &[(&str, String)]) {
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
pub(crate) fn load_plugin(name: &str) -> (MockHost, PathBuf) {
    load_plugin_from(plugin_folder(name))
}

/// Loads the plugin with the radio sounds and the HRIR sphere of `assets/hrir/`, so HRTF is
/// on.
pub(crate) fn load_plugin_with_hrir(name: &str) -> (MockHost, PathBuf) {
    let folder = plugin_folder(name);
    let sphere = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../assets/hrir/IRC_1002_C.bin");
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
pub(crate) fn game_client() -> GameLinkClient {
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
pub(crate) fn sync(client: &GameLinkClient, text: &str) -> String {
    match client.sync_request(text.as_bytes()) {
        SyncOutcome::Reply(reply) => String::from_utf8(reply).unwrap(),
        other => panic!("no reply to {text:?}: {other:?}"),
    }
}

/// Sends the async message `text` (without the `~`).
pub(crate) fn send(client: &GameLinkClient, text: &str) {
    client.send_async(text.as_bytes()).unwrap();
}

/// Waits until `found` holds for the host's calls; panics with `what` after the timeout.
pub(crate) fn wait_for(host: &MockHost, what: &str, found: impl Fn(&[HostCall]) -> bool) {
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
pub(crate) fn wait_until(what: &str, condition: impl Fn() -> bool) {
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
pub(crate) fn sent_command(calls: &[HostCall], prefix: &str) -> bool {
    calls.iter().any(|call| {
    matches!(call, HostCall::SendPluginCommand { command, target: SentCommandTarget::CurrentChannel, .. }
        if command.starts_with(prefix))
})
}

/// The values the plugin set for `property` on itself, in order.
pub(crate) fn own_int_values(calls: &[HostCall], property: ClientProperty) -> Vec<i32> {
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
pub(crate) fn own_text_values(calls: &[HostCall], property: ClientProperty) -> Vec<String> {
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
pub(crate) fn played(calls: &[HostCall], folder: &Path, sound: &str) -> bool {
    let expected = folder.join(format!("{sound}.wav"));
    calls
        .iter()
        .any(|call| matches!(call, HostCall::PlayWaveFile { path, .. } if *path == expected))
}

/// `frames` mono frames of a sine at `hertz`, amplitude `level`, starting at frame `start`.
pub(crate) fn sine(hertz: f64, level: f64, start: usize, frames: usize) -> Vec<i16> {
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
pub(crate) fn rms(samples: &[i16]) -> f64 {
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
pub(crate) fn position(nickname: &str, position: [f64; 3]) -> String {
    format!(
        "POS\t{nickname}\t[{},{},{}]\t[0,1,0]\ttrue\ttrue\ttrue\tfalse\tno\t0\t1\t0\tfalse\tfalse",
        position[0], position[1], position[2]
    )
}

/// Connects the server with the serious channel and two other players in it.
pub(crate) fn connect_three_clients(host: &MockHost) -> (ClientId, ClientId, ClientId) {
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
