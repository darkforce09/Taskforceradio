//! The SDK calls for 3D sound, the pre-processor, host paths, the log and the chat tab.
//!
//! **Role:** `systemset3DSettings`, `systemset3DListenerAttributes`, `channelset3DAttributes`,
//! `get/setPreProcessorConfigValue`, `getPluginPath` (the three-argument form of API 21 and
//! later), `getConfigPath`, `logMessage` and `printMessageToCurrentTab` as safe [`Ts3`] methods.
//!
//! **Position:** a method group of `ts3_functions`, over `handle` and `owned_memory`; `logging`
//! calls `log_message` from its logger thread.
//!
//! **Signals and state:** no state. None of these may run on an audio thread; the logging layer
//! queues audio-thread events for its own thread instead.
//!
//! **Invariants:** a path is read from a zeroed buffer up to its first NUL; text with a NUL byte is
//! refused before TeamSpeak sees it.

use std::ffi::c_char;
use std::path::PathBuf;

use super::handle::Ts3;
use super::owned_memory::path_from_buffer;
use crate::{ClientId, LogLevel, ServerConnectionHandlerId, Ts3Error, Ts3Vector};

/// The size of the buffer the path functions write into: Linux's `PATH_MAX`.
const PATH_BUFFER_SIZE: usize = 4096;

impl Ts3 {
    /// `systemset3DSettings`: the distance factor and rolloff scale of TeamSpeak's 3D sound.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn system_set_3d_settings(
        &self,
        server: ServerConnectionHandlerId,
        distance_factor: f32,
        rolloff_scale: f32,
    ) -> Result<(), Ts3Error> {
        let code = (self.functions.systemset3d_settings)(server.0, distance_factor, rolloff_scale);
        self.check("systemset3DSettings", code)
    }

    /// `systemset3DListenerAttributes`: our listening position and orientation.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn system_set_3d_listener_attributes(
        &self,
        server: ServerConnectionHandlerId,
        position: Ts3Vector,
        forward: Ts3Vector,
        up: Ts3Vector,
    ) -> Result<(), Ts3Error> {
        let (position, forward, up) = (position.to_sdk(), forward.to_sdk(), up.to_sdk());
        let code = (self.functions.systemset3d_listener_attributes)(
            server.0,
            &raw const position,
            &raw const forward,
            &raw const up,
        );
        self.check("systemset3DListenerAttributes", code)
    }

    /// `channelset3DAttributes`: the 3D position of `client`'s voice.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn channel_set_3d_attributes(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
        position: Ts3Vector,
    ) -> Result<(), Ts3Error> {
        let position = position.to_sdk();
        let code =
            (self.functions.channelset3d_attributes)(server.0, client.0, &raw const position);
        self.check("channelset3DAttributes", code)
    }

    /// `getPreProcessorConfigValue`: a capture pre-processor setting such as `"vad"`.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::InteriorNul`] when `ident` holds a NUL byte; [`Ts3Error::Call`] when
    /// TeamSpeak refuses the call.
    pub fn pre_processor_config_value(
        &self,
        server: ServerConnectionHandlerId,
        ident: &str,
    ) -> Result<String, Ts3Error> {
        let function = "getPreProcessorConfigValue";
        let ident = Self::c_string(function, ident)?;
        let mut text: *mut c_char = std::ptr::null_mut();
        let code = (self.functions.get_pre_processor_config_value)(
            server.0,
            ident.as_ptr(),
            &raw mut text,
        );
        self.check(function, code)?;
        // SAFETY: on success TeamSpeak hands over a NUL-terminated string it allocated (or null);
        // ownership passes to `take_string`, which frees it with `freeMemory`.
        Ok(unsafe { self.take_string(text) })
    }

    /// `setPreProcessorConfigValue`: sets a capture pre-processor setting such as `"vad"`.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::InteriorNul`] when `ident` or `value` holds a NUL byte;
    /// [`Ts3Error::Call`] when TeamSpeak refuses the call.
    pub fn set_pre_processor_config_value(
        &self,
        server: ServerConnectionHandlerId,
        ident: &str,
        value: &str,
    ) -> Result<(), Ts3Error> {
        let function = "setPreProcessorConfigValue";
        let ident = Self::c_string(function, ident)?;
        let value = Self::c_string(function, value)?;
        let code = (self.functions.set_pre_processor_config_value)(
            server.0,
            ident.as_ptr(),
            value.as_ptr(),
        );
        self.check(function, code)
    }

    /// `getPluginPath(path, maxLen, pluginID)`: the folder TeamSpeak loads plugins from.
    ///
    /// Needs the registered plugin ID, so it works from the first callback after
    /// `ts3plugin_registerPluginID`.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::PluginIdNotRegistered`] before `ts3plugin_registerPluginID`;
    /// [`Ts3Error::EmptyPath`] when TeamSpeak writes no path.
    pub fn plugin_path(&self) -> Result<PathBuf, Ts3Error> {
        let function = "getPluginPath";
        let plugin_id = self.plugin_id_for(function)?;
        let mut buffer = vec![0_u8; PATH_BUFFER_SIZE];
        (self.functions.get_plugin_path)(
            buffer.as_mut_ptr().cast::<c_char>(),
            buffer.len(),
            plugin_id.as_ptr(),
        );
        non_empty_path(function, &buffer)
    }

    /// `getConfigPath`: TeamSpeak's configuration folder (the one holding `settings.db`).
    ///
    /// # Errors
    ///
    /// [`Ts3Error::EmptyPath`] when TeamSpeak writes no path.
    pub fn config_path(&self) -> Result<PathBuf, Ts3Error> {
        let mut buffer = vec![0_u8; PATH_BUFFER_SIZE];
        (self.functions.get_config_path)(buffer.as_mut_ptr().cast::<c_char>(), buffer.len());
        non_empty_path("getConfigPath", &buffer)
    }

    /// `logMessage`: one line in TeamSpeak's client log under `channel`, tied to `server`.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::InteriorNul`] when `text` or `channel` holds a NUL byte; [`Ts3Error::Call`]
    /// when TeamSpeak refuses the line.
    pub fn log_message(
        &self,
        text: &str,
        level: LogLevel,
        channel: &str,
        server: ServerConnectionHandlerId,
    ) -> Result<(), Ts3Error> {
        let function = "logMessage";
        let text = Self::c_string(function, text)?;
        let channel = Self::c_string(function, channel)?;
        let code =
            (self.functions.log_message)(text.as_ptr(), level.to_sdk(), channel.as_ptr(), server.0);
        self.check(function, code)
    }

    /// `printMessageToCurrentTab`: a line of BB-code text in the chat of the tab in front.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::InteriorNul`] when `text` holds a NUL byte.
    pub fn print_message_to_current_tab(&self, text: &str) -> Result<(), Ts3Error> {
        let text = Self::c_string("printMessageToCurrentTab", text)?;
        (self.functions.print_message_to_current_tab)(text.as_ptr());
        Ok(())
    }
}

/// The path in `buffer`, or [`Ts3Error::EmptyPath`] when TeamSpeak wrote none.
fn non_empty_path(function: &'static str, buffer: &[u8]) -> Result<PathBuf, Ts3Error> {
    let path = path_from_buffer(buffer);
    if path.as_os_str().is_empty() {
        return Err(Ts3Error::EmptyPath { function });
    }
    Ok(path)
}
