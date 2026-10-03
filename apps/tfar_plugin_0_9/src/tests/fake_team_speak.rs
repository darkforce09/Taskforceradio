//! A recording fake of TeamSpeak for the unit tests: one server with channels and clients, every
//! side effect logged, and a core built over it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use tfar_ts3_host::{ChannelId, ClientId, ServerConnectionHandlerId, Ts3Error};

use crate::plugin_core::{PluginCore, PluginResources};
use crate::team_speak::{TeamSpeakActions, TeamSpeakQueries};

/// The fake's one server tab.
pub(crate) const SERVER: ServerConnectionHandlerId = ServerConnectionHandlerId(1);

/// Our own client on the fake server.
pub(crate) const ME: ClientId = ClientId(1);

/// The channel everyone starts in.
pub(crate) const LOBBY: ChannelId = ChannelId(1);

/// The second channel, named [`SERIOUS_CHANNEL_NAME`].
pub(crate) const SERIOUS: ChannelId = ChannelId(2);

/// The name of [`SERIOUS`].
pub(crate) const SERIOUS_CHANNEL_NAME: &str = "Serious";

/// A metadata block that says the client's plugin is connected and playing.
pub(crate) const PLAYING_METADATA: &str =
    "<TFAR>[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y[/B] [I]P:[/I][B]0.9.12[/B]</TFAR>";

/// One client of the fake server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FakeClient {
    pub(crate) nickname: String,
    pub(crate) channel: ChannelId,
    pub(crate) metadata: String,
    pub(crate) is_talking: bool,
}

/// The fake server's state.
#[derive(Clone, Debug)]
pub(crate) struct FakeWorld {
    pub(crate) is_connected: bool,
    pub(crate) clients: BTreeMap<ClientId, FakeClient>,
    pub(crate) channels: BTreeMap<ChannelId, String>,
    pub(crate) is_voice_activation_on: bool,
    pub(crate) refuses_mutes: bool,
}

/// One side effect the plugin caused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FakeAction {
    Mute(Vec<ClientId>),
    Unmute(Vec<ClientId>),
    Nickname(String),
    Metadata(String),
    PluginCommand(String),
    PlayWave(PathBuf),
    Move {
        channel: ChannelId,
        password: String,
    },
    Neutralise3d,
    CentreListener,
    CentreClient(ClientId),
    VoiceActivation(bool),
    InputActive(bool),
    Print(String),
}

/// The recording fake.
#[derive(Debug)]
pub(crate) struct FakeTeamSpeak {
    world: Mutex<FakeWorld>,
    actions: Mutex<Vec<FakeAction>>,
}

impl FakeTeamSpeak {
    /// A connected server with the channels `Lobby` and `Serious`, us (`Mock Player`) in the
    /// lobby and each of `others` there too, ids from 2, all with playing metadata.
    pub(crate) fn with_players(others: &[&str]) -> Arc<Self> {
        let mut clients = BTreeMap::new();
        for (id, nickname) in
            (1_u16..).zip(std::iter::once("Mock Player").chain(others.iter().copied()))
        {
            clients.insert(
                ClientId(id),
                FakeClient {
                    nickname: nickname.to_owned(),
                    channel: LOBBY,
                    metadata: if id == 1 {
                        String::new()
                    } else {
                        PLAYING_METADATA.to_owned()
                    },
                    is_talking: false,
                },
            );
        }
        let channels = BTreeMap::from([
            (LOBBY, "Lobby".to_owned()),
            (SERIOUS, SERIOUS_CHANNEL_NAME.to_owned()),
        ]);
        Arc::new(Self {
            world: Mutex::new(FakeWorld {
                is_connected: true,
                clients,
                channels,
                is_voice_activation_on: false,
                refuses_mutes: false,
            }),
            actions: Mutex::new(Vec::new()),
        })
    }

    /// The fake server's state, to read or change.
    pub(crate) fn world(&self) -> MutexGuard<'_, FakeWorld> {
        self.world.lock().unwrap()
    }

    /// Every side effect so far, and forgets them.
    pub(crate) fn take_actions(&self) -> Vec<FakeAction> {
        std::mem::take(&mut *self.actions.lock().unwrap())
    }

    /// Moves every client into `channel`.
    pub(crate) fn move_everyone(&self, channel: ChannelId) {
        for client in self.world().clients.values_mut() {
            client.channel = channel;
        }
    }

    fn record(&self, action: FakeAction) {
        self.actions.lock().unwrap().push(action);
    }

    fn client(&self, client: ClientId) -> Option<FakeClient> {
        self.world().clients.get(&client).cloned()
    }
}

impl TeamSpeakQueries for FakeTeamSpeak {
    fn current_server(&self) -> ServerConnectionHandlerId {
        SERVER
    }

    fn connected_servers(&self) -> Vec<ServerConnectionHandlerId> {
        if self.world().is_connected {
            vec![SERVER]
        } else {
            Vec::new()
        }
    }

    fn is_connected(&self, server: ServerConnectionHandlerId) -> bool {
        server == SERVER && self.world().is_connected
    }

    fn my_client_id(&self, server: ServerConnectionHandlerId) -> Option<ClientId> {
        self.is_connected(server).then_some(ME)
    }

    fn channel_of_client(
        &self,
        _: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Option<ChannelId> {
        self.client(client).map(|client| client.channel)
    }

    fn channel_name(&self, _: ServerConnectionHandlerId, channel: ChannelId) -> Option<String> {
        self.world().channels.get(&channel).cloned()
    }

    fn channel_list(&self, _: ServerConnectionHandlerId) -> Vec<ChannelId> {
        self.world().channels.keys().copied().collect()
    }

    fn channel_clients(&self, _: ServerConnectionHandlerId, channel: ChannelId) -> Vec<ClientId> {
        self.world()
            .clients
            .iter()
            .filter(|(_, client)| client.channel == channel)
            .map(|(id, _)| *id)
            .collect()
    }

    fn client_nickname(&self, _: ServerConnectionHandlerId, client: ClientId) -> Option<String> {
        self.client(client).map(|client| client.nickname)
    }

    fn client_metadata(&self, _: ServerConnectionHandlerId, client: ClientId) -> Option<String> {
        self.client(client).map(|client| client.metadata)
    }

    fn is_client_talking(&self, _: ServerConnectionHandlerId, client: ClientId) -> bool {
        self.client(client).is_some_and(|client| client.is_talking)
    }

    fn server_name(&self, _: ServerConnectionHandlerId) -> Option<String> {
        Some("Fake Server".to_owned())
    }

    fn is_voice_activation_on(&self, _: ServerConnectionHandlerId) -> bool {
        self.world().is_voice_activation_on
    }
}

/// A refusal as TeamSpeak gives it.
fn refusal(function: &'static str) -> Ts3Error {
    Ts3Error::Call {
        function,
        code: 0x0200,
        message: "refused by the fake".to_owned(),
    }
}

impl TeamSpeakActions for FakeTeamSpeak {
    fn mute_clients(
        &self,
        _: ServerConnectionHandlerId,
        clients: &[ClientId],
    ) -> Result<(), Ts3Error> {
        if clients.is_empty() {
            return Ok(());
        }
        self.record(FakeAction::Mute(clients.to_vec()));
        if self.world().refuses_mutes {
            Err(refusal("requestMuteClients"))
        } else {
            Ok(())
        }
    }

    fn unmute_clients(
        &self,
        _: ServerConnectionHandlerId,
        clients: &[ClientId],
    ) -> Result<(), Ts3Error> {
        if !clients.is_empty() {
            self.record(FakeAction::Unmute(clients.to_vec()));
        }
        Ok(())
    }

    fn set_own_nickname(
        &self,
        _: ServerConnectionHandlerId,
        nickname: &str,
    ) -> Result<(), Ts3Error> {
        self.record(FakeAction::Nickname(nickname.to_owned()));
        if let Some(me) = self.world().clients.get_mut(&ME) {
            nickname.clone_into(&mut me.nickname);
        }
        Ok(())
    }

    fn set_own_metadata(
        &self,
        _: ServerConnectionHandlerId,
        metadata: &str,
    ) -> Result<(), Ts3Error> {
        self.record(FakeAction::Metadata(metadata.to_owned()));
        if let Some(me) = self.world().clients.get_mut(&ME) {
            metadata.clone_into(&mut me.metadata);
        }
        Ok(())
    }

    fn send_plugin_command(
        &self,
        _: ServerConnectionHandlerId,
        command: &str,
    ) -> Result<(), Ts3Error> {
        self.record(FakeAction::PluginCommand(command.to_owned()));
        Ok(())
    }

    fn play_wave_file(&self, _: ServerConnectionHandlerId, path: &Path) -> Result<(), Ts3Error> {
        self.record(FakeAction::PlayWave(path.to_owned()));
        Ok(())
    }

    fn move_client(
        &self,
        _: ServerConnectionHandlerId,
        _: ClientId,
        channel: ChannelId,
        password: &str,
    ) -> Result<(), Ts3Error> {
        self.record(FakeAction::Move {
            channel,
            password: password.to_owned(),
        });
        Ok(())
    }

    fn neutralise_3d_settings(&self, _: ServerConnectionHandlerId) -> Result<(), Ts3Error> {
        self.record(FakeAction::Neutralise3d);
        Ok(())
    }

    fn centre_listener(&self, _: ServerConnectionHandlerId) -> Result<(), Ts3Error> {
        self.record(FakeAction::CentreListener);
        Ok(())
    }

    fn centre_client(
        &self,
        _: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Result<(), Ts3Error> {
        self.record(FakeAction::CentreClient(client));
        Ok(())
    }

    fn set_voice_activation(
        &self,
        _: ServerConnectionHandlerId,
        is_on: bool,
    ) -> Result<(), Ts3Error> {
        self.record(FakeAction::VoiceActivation(is_on));
        self.world().is_voice_activation_on = is_on;
        Ok(())
    }

    fn set_input_active(
        &self,
        _: ServerConnectionHandlerId,
        is_active: bool,
    ) -> Result<(), Ts3Error> {
        self.record(FakeAction::InputActive(is_active));
        Ok(())
    }

    fn print_message(&self, text: &str) {
        self.record(FakeAction::Print(text.to_owned()));
    }
}

/// The folder the fake core looks for radio sounds in: one that does not exist.
pub(crate) fn missing_plugin_folder() -> PathBuf {
    PathBuf::from("/nonexistent/tfar-plugin-0-9-unit-tests")
}

/// A core over `fake` without radio sounds or HRIR sphere and with no push-to-talk delay, with
/// the fake server's state made.
pub(crate) fn core_over(fake: &Arc<FakeTeamSpeak>) -> PluginCore {
    core_with_delay(fake, Duration::ZERO)
}

/// [`core_over`] with a push-to-talk delay of `delay`.
pub(crate) fn core_with_delay(fake: &Arc<FakeTeamSpeak>, delay: Duration) -> PluginCore {
    let core = PluginCore::new(
        Arc::clone(fake) as Arc<dyn crate::team_speak::TeamSpeak>,
        PluginResources {
            plugin_folder: missing_plugin_folder(),
            hrir: None,
            push_to_talk_delay: delay,
        },
        Instant::now(),
    );
    core.add_connection(SERVER);
    core
}

/// The plugin commands among `actions`.
pub(crate) fn plugin_commands(actions: &[FakeAction]) -> Vec<String> {
    actions
        .iter()
        .filter_map(|action| match action {
            FakeAction::PluginCommand(command) => Some(command.clone()),
            _ => None,
        })
        .collect()
}

/// Whether `actions` hold anything a dormant plugin must not do.
pub(crate) fn has_dormant_forbidden_action(actions: &[FakeAction]) -> bool {
    actions.iter().any(|action| {
        matches!(
            action,
            FakeAction::Mute(_)
                | FakeAction::Unmute(_)
                | FakeAction::Nickname(_)
                | FakeAction::Metadata(_)
                | FakeAction::PluginCommand(_)
                | FakeAction::Move { .. }
        )
    })
}
