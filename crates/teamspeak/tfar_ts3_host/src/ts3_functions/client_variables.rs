//! The SDK variable calls: client, own-client, channel and server variables, read and written.
//!
//! **Role:** typed property access over `get/setClient*Variable*`, `getChannelVariableAsString`,
//! `getServerVariableAsString` and `flushClientSelfUpdates`.
//!
//! **Position:** a method group of `ts3_functions`, over `handle` and `owned_memory`.
//!
//! **Signals and state:** no state; called from TeamSpeak's main thread or plugin threads.
//!
//! **Invariants:** every string TeamSpeak returns is copied and freed with `freeMemory`; a text
//! argument with a NUL byte is refused before TeamSpeak sees it.

use std::ffi::{c_char, c_int};

use super::handle::Ts3;
use crate::{
    ChannelId, ChannelProperty, ClientId, ClientProperty, ServerConnectionHandlerId,
    ServerProperty, Ts3Error,
};

impl Ts3 {
    /// `getClientVariableAsString`: a string property of `client`.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn client_variable_as_string(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
        property: ClientProperty,
    ) -> Result<String, Ts3Error> {
        let mut text: *mut c_char = std::ptr::null_mut();
        let code = (self.functions.get_client_variable_as_string)(
            server.0,
            client.0,
            property.flag(),
            &raw mut text,
        );
        self.check("getClientVariableAsString", code)?;
        // SAFETY: on success TeamSpeak hands over a NUL-terminated string it allocated (or null);
        // ownership passes to `take_string`, which frees it with `freeMemory`.
        Ok(unsafe { self.take_string(text) })
    }

    /// `getClientVariableAsInt`: an integer property of `client`.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn client_variable_as_int(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
        property: ClientProperty,
    ) -> Result<i32, Ts3Error> {
        let mut value: c_int = 0;
        let code = (self.functions.get_client_variable_as_int)(
            server.0,
            client.0,
            property.flag(),
            &raw mut value,
        );
        self.check("getClientVariableAsInt", code)?;
        Ok(value)
    }

    /// `getClientSelfVariableAsInt`: an integer property of our own client.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn client_self_variable_as_int(
        &self,
        server: ServerConnectionHandlerId,
        property: ClientProperty,
    ) -> Result<i32, Ts3Error> {
        let mut value: c_int = 0;
        let code = (self.functions.get_client_self_variable_as_int)(
            server.0,
            property.flag(),
            &raw mut value,
        );
        self.check("getClientSelfVariableAsInt", code)?;
        Ok(value)
    }

    /// `setClientSelfVariableAsString`: stages a string property of our own client; it reaches the
    /// server with [`Ts3::flush_client_self_updates`].
    ///
    /// # Errors
    ///
    /// [`Ts3Error::InteriorNul`] when `value` holds a NUL byte; [`Ts3Error::Call`] when
    /// TeamSpeak refuses the call.
    pub fn set_client_self_variable_as_string(
        &self,
        server: ServerConnectionHandlerId,
        property: ClientProperty,
        value: &str,
    ) -> Result<(), Ts3Error> {
        let function = "setClientSelfVariableAsString";
        let value = Self::c_string(function, value)?;
        let code = (self.functions.set_client_self_variable_as_string)(
            server.0,
            property.flag(),
            value.as_ptr(),
        );
        self.check(function, code)
    }

    /// `setClientSelfVariableAsInt`: stages an integer property of our own client.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn set_client_self_variable_as_int(
        &self,
        server: ServerConnectionHandlerId,
        property: ClientProperty,
        value: i32,
    ) -> Result<(), Ts3Error> {
        let code =
            (self.functions.set_client_self_variable_as_int)(server.0, property.flag(), value);
        self.check("setClientSelfVariableAsInt", code)
    }

    /// `flushClientSelfUpdates`: sends the staged own-client changes, with no return code.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn flush_client_self_updates(
        &self,
        server: ServerConnectionHandlerId,
    ) -> Result<(), Ts3Error> {
        let code = (self.functions.flush_client_self_updates)(server.0, std::ptr::null());
        self.check("flushClientSelfUpdates", code)
    }

    /// `getChannelVariableAsString`: a string property of `channel`.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn channel_variable_as_string(
        &self,
        server: ServerConnectionHandlerId,
        channel: ChannelId,
        property: ChannelProperty,
    ) -> Result<String, Ts3Error> {
        let mut text: *mut c_char = std::ptr::null_mut();
        let code = (self.functions.get_channel_variable_as_string)(
            server.0,
            channel.0,
            property.flag(),
            &raw mut text,
        );
        self.check("getChannelVariableAsString", code)?;
        // SAFETY: on success TeamSpeak hands over a NUL-terminated string it allocated (or null);
        // ownership passes to `take_string`, which frees it with `freeMemory`.
        Ok(unsafe { self.take_string(text) })
    }

    /// `getServerVariableAsString`: a string property of the virtual server behind `server`.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn server_variable_as_string(
        &self,
        server: ServerConnectionHandlerId,
        property: ServerProperty,
    ) -> Result<String, Ts3Error> {
        let mut text: *mut c_char = std::ptr::null_mut();
        let code = (self.functions.get_server_variable_as_string)(
            server.0,
            property.flag(),
            &raw mut text,
        );
        self.check("getServerVariableAsString", code)?;
        // SAFETY: on success TeamSpeak hands over a NUL-terminated string it allocated (or null);
        // ownership passes to `take_string`, which frees it with `freeMemory`.
        Ok(unsafe { self.take_string(text) })
    }
}
