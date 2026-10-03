//! Unit tests for the plugin's metadata and its state before `init`.

use super::*;

#[test]
fn the_metadata_is_the_original_plugins() {
    assert_eq!(TfarPlugin::NAME, c"Task Force Arma 3 Radio");
    assert_eq!(TfarPlugin::VERSION, c"0.9.12");
    assert_eq!(TfarPlugin::AUTHOR, c"[TF]Nkey");
    assert_eq!(TfarPlugin::DESCRIPTION, c"Radio Addon for Arma 3");
    assert_eq!(TfarPlugin::INFO_TITLE, c"Task Force Radio Status (0.9.12)");
    assert_eq!(TfarPlugin::COMMAND_KEYWORD, c"");
}

#[test]
fn a_new_plugin_runs_nothing_and_shutdown_is_harmless() {
    let plugin = TfarPlugin::new();
    assert!(!plugin.is_running());
    plugin.shutdown();
    assert!(!plugin.is_running());
}

#[test]
fn the_plugin_serves_the_0_9_flavour() {
    assert_eq!(FLAVOUR, Flavour::V0_9);
}

#[test]
fn every_chat_command_is_answered_handled_as_the_original_answers_zero() {
    let plugin = TfarPlugin::new();
    assert_eq!(
        plugin.process_command(ServerConnectionHandlerId(1), "anything"),
        CommandHandling::Handled
    );
}
