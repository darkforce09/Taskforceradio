//! The SDK requests that change something: moving and muting clients, plugin commands to other
//! clients, and wave playback.
//!
//! **Role:** `requestClientMove`, `requestMuteClients`, `requestUnmuteClients`,
//! `sendPluginCommand` and `playWaveFile` as safe [`Ts3`] methods.
//!
//! **Position:** a method group of `ts3_functions`, over `handle`.
//!
//! **Signals and state:** no state; called from TeamSpeak's main thread or plugin threads.
//!
//! **Invariants:** every client array handed to TeamSpeak ends in the `0` terminator it reads up
//! to; requests carry no return code, so TeamSpeak reports their server answer through
//! `onServerErrorEvent` without one.

use std::path::Path;

use super::handle::Ts3;
use crate::{ChannelId, ClientId, PluginCommandTarget, ServerConnectionHandlerId, Ts3Error};

impl Ts3 {
    /// `requestClientMove`: asks the server to move `client` into `channel`; `password` is the
    /// channel password, empty for none.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::InteriorNul`] when `password` holds a NUL byte; [`Ts3Error::Call`] when
    /// TeamSpeak refuses the request.
    pub fn request_client_move(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
        channel: ChannelId,
        password: &str,
    ) -> Result<(), Ts3Error> {
        let function = "requestClientMove";
        let password = Self::c_string(function, password)?;
        let code = (self.functions.request_client_move)(
            server.0,
            client.0,
            channel.0,
            password.as_ptr(),
            std::ptr::null(),
        );
        self.check(function, code)
    }

    /// `requestMuteClients`: mutes `clients` locally; an empty slice asks nothing of TeamSpeak.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn request_mute_clients(
        &self,
        server: ServerConnectionHandlerId,
        clients: &[ClientId],
    ) -> Result<(), Ts3Error> {
        if clients.is_empty() {
            return Ok(());
        }
        let terminated = terminated_client_array(clients);
        let code =
            (self.functions.request_mute_clients)(server.0, terminated.as_ptr(), std::ptr::null());
        self.check("requestMuteClients", code)
    }

    /// `requestUnmuteClients`: unmutes `clients` locally; an empty slice asks nothing of
    /// TeamSpeak.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn request_unmute_clients(
        &self,
        server: ServerConnectionHandlerId,
        clients: &[ClientId],
    ) -> Result<(), Ts3Error> {
        if clients.is_empty() {
            return Ok(());
        }
        let terminated = terminated_client_array(clients);
        // The `ts3plugin-sys` field name carries the spelling `requset`; the SDK name is
        // `requestUnmuteClients` and the slot is checked by the layout test.
        let code = (self.functions.requset_unmute_clients)(
            server.0,
            terminated.as_ptr(),
            std::ptr::null(),
        );
        self.check("requestUnmuteClients", code)
    }

    /// `sendPluginCommand`: sends `command` to the plugins with our ID on the `target` clients.
    ///
    /// TeamSpeak reports nothing back for this call, so its own failures go unseen.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::PluginIdNotRegistered`] before `ts3plugin_registerPluginID`;
    /// [`Ts3Error::InteriorNul`] when `command` holds a NUL byte.
    pub fn send_plugin_command(
        &self,
        server: ServerConnectionHandlerId,
        command: &str,
        target: PluginCommandTarget<'_>,
    ) -> Result<(), Ts3Error> {
        let function = "sendPluginCommand";
        let plugin_id = self.plugin_id_for(function)?;
        let command = Self::c_string(function, command)?;
        let targets = match target {
            PluginCommandTarget::Clients(clients) => Some(terminated_client_array(clients)),
            _ => None,
        };
        let target_pointer = targets.as_ref().map_or(std::ptr::null(), Vec::as_ptr);
        (self.functions.send_plugin_command)(
            server.0,
            plugin_id.as_ptr(),
            command.as_ptr(),
            target.mode(),
            target_pointer,
            std::ptr::null(),
        );
        Ok(())
    }

    /// `playWaveFile`: plays the WAV file at `path` on the playback device of `server`.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::InteriorNul`] when `path` holds a NUL byte; [`Ts3Error::Call`] when
    /// TeamSpeak cannot play the file.
    pub fn play_wave_file(
        &self,
        server: ServerConnectionHandlerId,
        path: &Path,
    ) -> Result<(), Ts3Error> {
        let function = "playWaveFile";
        let path = Self::c_string_from_bytes(function, path.as_os_str().as_encoded_bytes())?;
        let code = (self.functions.play_wave_file)(server.0, path.as_ptr());
        self.check(function, code)
    }

    /// Converts raw `bytes` (a path) to a C string for `function`, or reports its NUL byte.
    fn c_string_from_bytes(
        function: &'static str,
        bytes: &[u8],
    ) -> Result<std::ffi::CString, Ts3Error> {
        std::ffi::CString::new(bytes).map_err(|error| Ts3Error::InteriorNul {
            function,
            position: error.nul_position(),
        })
    }
}

/// `clients` as the `anyID` array TeamSpeak reads, with the `0` terminator appended.
fn terminated_client_array(clients: &[ClientId]) -> Vec<u16> {
    clients
        .iter()
        .map(|client| client.0)
        .chain(std::iter::once(0))
        .collect()
}
