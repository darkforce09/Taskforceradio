//! The SDK queries for connections, clients and channels: which server tab is current, who we are,
//! who is where.
//!
//! **Role:** the read-only topology calls of the table, each as a safe [`Ts3`] method.
//!
//! **Position:** a method group of `ts3_functions`, over `handle` and `owned_memory`.
//!
//! **Signals and state:** no state; TeamSpeak expects these calls on its main thread or a plugin
//! thread, never on an audio thread.
//!
//! **Invariants:** every list TeamSpeak returns is read up to its `0` terminator and freed.

use std::ffi::c_int;

use super::handle::Ts3;
use crate::{ChannelId, ClientId, ConnectStatus, ServerConnectionHandlerId, Ts3Error};

impl Ts3 {
    /// `getCurrentServerConnectionHandlerID`: the server tab in front; `0` when there is none.
    pub fn current_server_connection_handler_id(&self) -> ServerConnectionHandlerId {
        ServerConnectionHandlerId((self.functions.get_current_server_connection_handler_id)())
    }

    /// `getServerConnectionHandlerList`: every server tab.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn server_connection_handler_list(
        &self,
    ) -> Result<Vec<ServerConnectionHandlerId>, Ts3Error> {
        let mut list: *mut u64 = std::ptr::null_mut();
        let code = (self.functions.get_server_connection_handler_list)(&raw mut list);
        self.check("getServerConnectionHandlerList", code)?;
        // SAFETY: on success TeamSpeak hands over a 0-terminated `uint64` array it allocated (or
        // null); ownership passes to `take_terminated_list`, which frees it.
        let ids = unsafe { self.take_terminated_list(list) };
        Ok(ids.into_iter().map(ServerConnectionHandlerId).collect())
    }

    /// `getConnectionStatus`: how far the connection of `server` has come.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn connection_status(
        &self,
        server: ServerConnectionHandlerId,
    ) -> Result<ConnectStatus, Ts3Error> {
        let mut status: c_int = 0;
        let code = (self.functions.get_connection_status)(server.0, &raw mut status);
        self.check("getConnectionStatus", code)?;
        Ok(ConnectStatus::from_raw(status))
    }

    /// `getClientID`: our own client ID on `server`.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn client_id(&self, server: ServerConnectionHandlerId) -> Result<ClientId, Ts3Error> {
        let mut client: u16 = 0;
        let code = (self.functions.get_client_id)(server.0, &raw mut client);
        self.check("getClientID", code)?;
        Ok(ClientId(client))
    }

    /// `getClientList`: every client visible on `server`.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn client_list(
        &self,
        server: ServerConnectionHandlerId,
    ) -> Result<Vec<ClientId>, Ts3Error> {
        let mut list: *mut u16 = std::ptr::null_mut();
        let code = (self.functions.get_client_list)(server.0, &raw mut list);
        self.check("getClientList", code)?;
        // SAFETY: on success TeamSpeak hands over a 0-terminated `anyID` array it allocated (or
        // null); ownership passes to `take_terminated_list`, which frees it.
        let ids = unsafe { self.take_terminated_list(list) };
        Ok(ids.into_iter().map(ClientId).collect())
    }

    /// `getChannelList`: every channel on `server`.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn channel_list(
        &self,
        server: ServerConnectionHandlerId,
    ) -> Result<Vec<ChannelId>, Ts3Error> {
        let mut list: *mut u64 = std::ptr::null_mut();
        let code = (self.functions.get_channel_list)(server.0, &raw mut list);
        self.check("getChannelList", code)?;
        // SAFETY: on success TeamSpeak hands over a 0-terminated `uint64` array it allocated (or
        // null); ownership passes to `take_terminated_list`, which frees it.
        let ids = unsafe { self.take_terminated_list(list) };
        Ok(ids.into_iter().map(ChannelId).collect())
    }

    /// `getChannelClientList`: the clients in `channel` on `server`.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn channel_client_list(
        &self,
        server: ServerConnectionHandlerId,
        channel: ChannelId,
    ) -> Result<Vec<ClientId>, Ts3Error> {
        let mut list: *mut u16 = std::ptr::null_mut();
        let code = (self.functions.get_channel_client_list)(server.0, channel.0, &raw mut list);
        self.check("getChannelClientList", code)?;
        // SAFETY: on success TeamSpeak hands over a 0-terminated `anyID` array it allocated (or
        // null); ownership passes to `take_terminated_list`, which frees it.
        let ids = unsafe { self.take_terminated_list(list) };
        Ok(ids.into_iter().map(ClientId).collect())
    }

    /// `getChannelOfClient`: the channel `client` is in on `server`.
    ///
    /// # Errors
    ///
    /// [`Ts3Error::Call`] with TeamSpeak's code and text when TeamSpeak refuses the call.
    pub fn channel_of_client(
        &self,
        server: ServerConnectionHandlerId,
        client: ClientId,
    ) -> Result<ChannelId, Ts3Error> {
        let mut channel: u64 = 0;
        let code = (self.functions.get_channel_of_client)(server.0, client.0, &raw mut channel);
        self.check("getChannelOfClient", code)?;
        Ok(ChannelId(channel))
    }
}
