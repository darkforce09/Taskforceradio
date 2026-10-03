//! Unit tests for the push-to-talk and its release thread, over a fake of TeamSpeak that records
//! the microphone switches and the broadcasts.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tfar_ts3_host::{ChannelId, ClientId};

use super::*;

/// The connection every test presses on.
const SERVER: ServerConnectionHandlerId = ServerConnectionHandlerId(1);

/// The press broadcast.
const PRESSED: &str = "TANGENT\tPRESSED\t30.0\t20000\tdigital\ttf_anprc152_1\t1\t0\tAlpha";

/// The release broadcast.
const RELEASED: &str = "TANGENT\tRELEASED\t30.0\t20000\tdigital\ttf_anprc152_1\t1\t0\tAlpha";

/// The release delay of every test.
const DELAY: Duration = Duration::from_millis(150);

/// What the fake was told to do.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Recorded {
    InputDeactivated(bool),
    VoiceActivation(bool),
    Broadcast(String),
}

/// TeamSpeak with voice activation off, recording the push-to-talk's side effects.
#[derive(Default)]
struct RecordingTeamSpeak {
    recorded: Mutex<Vec<Recorded>>,
}

impl RecordingTeamSpeak {
    /// A fake that lives as long as the release thread may use it.
    fn leaked() -> &'static Self {
        Box::leak(Box::default())
    }

    fn broadcasts(&self) -> Vec<String> {
        self.recorded
            .lock()
            .iter()
            .filter_map(|recorded| match recorded {
                Recorded::Broadcast(command) => Some(command.clone()),
                _ => None,
            })
            .collect()
    }
}

impl TeamSpeakActions for RecordingTeamSpeak {
    fn current_server(&self) -> ServerConnectionHandlerId {
        SERVER
    }
    fn is_connected(&self, _: ServerConnectionHandlerId) -> bool {
        true
    }
    fn own_client_id(&self, _: ServerConnectionHandlerId) -> Option<ClientId> {
        None
    }
    fn client_nickname(&self, _: ServerConnectionHandlerId, _: ClientId) -> Option<String> {
        None
    }
    fn client_metadata(&self, _: ServerConnectionHandlerId, _: ClientId) -> Option<String> {
        None
    }
    fn channel_of_client(&self, _: ServerConnectionHandlerId, _: ClientId) -> Option<ChannelId> {
        None
    }
    fn channel_clients(&self, _: ServerConnectionHandlerId, _: ChannelId) -> Vec<ClientId> {
        Vec::new()
    }
    fn channels(&self, _: ServerConnectionHandlerId) -> Vec<(ChannelId, String)> {
        Vec::new()
    }
    fn server_name(&self, _: ServerConnectionHandlerId) -> Option<String> {
        None
    }
    fn server_unique_identifier(&self, _: ServerConnectionHandlerId) -> Option<String> {
        None
    }
    fn channel_name(&self, _: ServerConnectionHandlerId, _: ChannelId) -> Option<String> {
        None
    }
    fn plugin_path(&self) -> Option<PathBuf> {
        None
    }
    fn config_path(&self) -> Option<PathBuf> {
        None
    }
    fn is_voice_activation_enabled(&self, _: ServerConnectionHandlerId) -> bool {
        false
    }
    fn mute_clients(&self, _: ServerConnectionHandlerId, _: &[ClientId]) {}
    fn unmute_clients(&self, _: ServerConnectionHandlerId, _: &[ClientId]) {}
    fn set_own_nickname(&self, _: ServerConnectionHandlerId, _: &str) -> bool {
        false
    }
    fn set_own_metadata(&self, _: ServerConnectionHandlerId, _: &str) {}
    fn send_plugin_command(
        &self,
        _: ServerConnectionHandlerId,
        command: &str,
        _: PluginCommandTarget<'_>,
    ) {
        self.recorded
            .lock()
            .push(Recorded::Broadcast(command.to_owned()));
    }
    fn play_wave_file(&self, _: ServerConnectionHandlerId, _: &Path) {}
    fn move_own_client(&self, _: ServerConnectionHandlerId, _: ChannelId, _: &str) {}
    fn neutralise_3d_settings(&self, _: ServerConnectionHandlerId) {}
    fn neutralise_listener(&self, _: ServerConnectionHandlerId) {}
    fn neutralise_client(&self, _: ServerConnectionHandlerId, _: ClientId) {}
    fn set_voice_activation(&self, _: ServerConnectionHandlerId, is_enabled: bool) {
        self.recorded
            .lock()
            .push(Recorded::VoiceActivation(is_enabled));
    }
    fn set_input_deactivated(&self, _: ServerConnectionHandlerId, is_deactivated: bool) {
        self.recorded
            .lock()
            .push(Recorded::InputDeactivated(is_deactivated));
    }
    fn print_to_current_tab(&self, _: &str) {}
}

/// Waits until `ts` has recorded a broadcast of `command`, at most 2 s.
fn wait_for_broadcast(ts: &RecordingTeamSpeak, command: &str) -> bool {
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(2) {
        if ts.broadcasts().iter().any(|sent| sent == command) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

#[test]
fn a_release_switches_the_microphone_off_and_broadcasts_after_its_delay() {
    let ts = RecordingTeamSpeak::leaked();
    let push_to_talk = PushToTalk::new();
    push_to_talk.press(ts, SERVER, PRESSED);
    push_to_talk.release(ts, SERVER, RELEASED.to_owned(), DELAY);
    assert_eq!(ts.broadcasts(), vec![PRESSED.to_owned()]);
    assert!(wait_for_broadcast(ts, RELEASED));
    assert_eq!(
        ts.recorded.lock().last(),
        Some(&Recorded::Broadcast(RELEASED.to_owned()))
    );
    assert!(
        ts.recorded
            .lock()
            .contains(&Recorded::InputDeactivated(true))
    );
    push_to_talk.stop();
}

#[test]
fn a_game_end_cancels_a_waiting_release_so_a_dormant_plugin_never_broadcasts_it() {
    let ts = RecordingTeamSpeak::leaked();
    let push_to_talk = PushToTalk::new();
    push_to_talk.press(ts, SERVER, PRESSED);
    push_to_talk.release(ts, SERVER, RELEASED.to_owned(), DELAY);
    push_to_talk.cancel_pending_releases();
    std::thread::sleep(DELAY.saturating_mul(3));
    assert_eq!(ts.broadcasts(), vec![PRESSED.to_owned()]);

    // The next game's press and release act as usual.
    push_to_talk.press(ts, SERVER, PRESSED);
    push_to_talk.release(ts, SERVER, RELEASED.to_owned(), DELAY);
    assert!(wait_for_broadcast(ts, RELEASED));
    assert_eq!(
        ts.broadcasts(),
        vec![PRESSED.to_owned(), PRESSED.to_owned(), RELEASED.to_owned()]
    );
    push_to_talk.stop();
}
