//! `TeamSpeakActions`: every side effect the plugin causes in TeamSpeak, as one trait.
//!
//! **Role:** name each TeamSpeak write of the 0.9 plugin: the local mutes, the nickname and the
//! metadata (each set and flushed), the plugin command to the channel, the wave playback, the
//! channel move, the 3D neutralising calls, voice activation and the microphone switch, and a
//! line in the chat tab.
//!
//! **Position:** a supertrait of [`crate::team_speak::TeamSpeak`]; implemented over the function
//! table by `Ts3TeamSpeak` and by the recording fake of the unit tests. The plugin decides under
//! its server lock and calls these after releasing it.
//!
//! **Signals and state:** none of its own. Called on TeamSpeak's main thread, the game link
//! thread, the service thread and the push-to-talk release thread; never on an audio thread and
//! never under a server lock.
//!
//! **Invariants:** a refused request comes back as the `Ts3Error`, so the caller can forget a
//! mute TeamSpeak did not make; nothing here panics.

use std::path::Path;

use tfar_ts3_host::{ChannelId, ClientId, ServerConnectionHandlerId, Ts3Error};

/// The TeamSpeak side effects of the 0.9 plugin.
pub trait TeamSpeakActions {
    /// Mutes `clients` locally (`requestMuteClients`); an empty list asks nothing.
    ///
    /// # Errors
    ///
    /// The [`Ts3Error`] of a refused request.
    fn mute_clients(
        &self,
        server: ServerConnectionHandlerId,
        clients: &[ClientId],
    ) -> Result<(), Ts3Error>;

    /// Unmutes `clients` locally (`requestUnmuteClients`); an empty list asks nothing.
    ///
    /// # Errors
    ///
    /// The [`Ts3Error`] of a refused request.
    fn unmute_clients(
        &self,
        server: ServerConnectionHandlerId,
        clients: &[ClientId],
    ) -> Result<(), Ts3Error>;

    /// Sets our nickname and flushes it to the server.
    ///
    /// # Errors
    ///
    /// The [`Ts3Error`] of the set or the flush.
    fn set_own_nickname(
        &self,
        server: ServerConnectionHandlerId,
        nickname: &str,
    ) -> Result<(), Ts3Error>;

    /// Sets our whole client metadata and flushes it to the server (`setMetaData`).
    ///
    /// # Errors
    ///
    /// The [`Ts3Error`] of the set or the flush.
    fn set_own_metadata(
        &self,
        server: ServerConnectionHandlerId,
        metadata: &str,
    ) -> Result<(), Ts3Error>;

    /// Sends `command` to the TFAR plugins of every client in our channel
    /// (`sendPluginCommand`, `PluginCommandTarget_CURRENT_CHANNEL`).
    ///
    /// # Errors
    ///
    /// The [`Ts3Error`] when the command cannot be handed over.
    fn send_plugin_command(
        &self,
        server: ServerConnectionHandlerId,
        command: &str,
    ) -> Result<(), Ts3Error>;

    /// Plays the WAV file at `path` through TeamSpeak (`playWaveFile`).
    ///
    /// # Errors
    ///
    /// The [`Ts3Error`] when TeamSpeak cannot play it.
    fn play_wave_file(
        &self,
        server: ServerConnectionHandlerId,
        path: &Path,
    ) -> Result<(), Ts3Error>;

    /// Asks the server to move `client` into `channel` with `password` (`requestClientMove`).
    ///
    /// # Errors
    ///
    /// The [`Ts3Error`] of a refused request.
    fn move_client(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
        channel: ChannelId,
        password: &str,
    ) -> Result<(), Ts3Error>;

    /// Sets TeamSpeak's own 3D sound to distance factor 1 and rolloff scale 1
    /// (`systemset3DSettings(1, 1)`), so the plugin's processing alone shapes the voices.
    ///
    /// # Errors
    ///
    /// The [`Ts3Error`] of a refused call.
    fn neutralise_3d_settings(&self, server: ServerConnectionHandlerId) -> Result<(), Ts3Error>;

    /// Puts TeamSpeak's 3D listener at the origin (`systemset3DListenerAttributes`).
    ///
    /// # Errors
    ///
    /// The [`Ts3Error`] of a refused call.
    fn centre_listener(&self, server: ServerConnectionHandlerId) -> Result<(), Ts3Error>;

    /// Puts `client`'s voice at the origin of TeamSpeak's 3D sound (`channelset3DAttributes`).
    ///
    /// # Errors
    ///
    /// The [`Ts3Error`] of a refused call.
    fn centre_client(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Result<(), Ts3Error>;

    /// Turns voice activation on or off (`setPreProcessorConfigValue("vad", "true"|"false")`).
    ///
    /// # Errors
    ///
    /// The [`Ts3Error`] of a refused call.
    fn set_voice_activation(
        &self,
        server: ServerConnectionHandlerId,
        is_on: bool,
    ) -> Result<(), Ts3Error>;

    /// Switches our microphone on (`INPUT_ACTIVE`) or off (`INPUT_DEACTIVATED`) through
    /// `CLIENT_INPUT_DEACTIVATED`, then flushes.
    ///
    /// # Errors
    ///
    /// The [`Ts3Error`] of the set or the flush.
    fn set_input_active(
        &self,
        server: ServerConnectionHandlerId,
        is_active: bool,
    ) -> Result<(), Ts3Error>;

    /// Prints `text` (BB-code allowed) in the chat tab in front.
    fn print_message(&self, text: &str);
}
