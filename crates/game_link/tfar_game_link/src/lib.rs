//! The localhost TCP link between an Arma extension and its TeamSpeak plugin: [`protocol`] (the
//! frames both sides exchange), [`server`] (the plugin's listener, which answers the extension) and
//! [`client`] (the extension's lazy, time-bounded connection).

pub mod client;
pub mod protocol;
pub mod server;
