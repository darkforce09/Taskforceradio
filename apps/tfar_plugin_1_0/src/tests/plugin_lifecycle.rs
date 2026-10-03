//! Unit tests for the plugin's metadata and its state before `init`.

use super::*;

#[test]
fn the_metadata_is_the_original_plugins() {
    assert_eq!(TfarPlugin::NAME, c"Task Force Arrowhead Radio");
    assert_eq!(TfarPlugin::VERSION, c"1.0.0.1");
    assert_eq!(TfarPlugin::AUTHOR, c"[TF]Nkey and Dedmen");
    assert_eq!(TfarPlugin::DESCRIPTION, c"Radio Addon for Arma 3");
    assert_eq!(TfarPlugin::INFO_TITLE, c"Task Force Radio Status");
    assert_eq!(TfarPlugin::COMMAND_KEYWORD, c"tfar");
}

#[test]
fn the_reported_version_is_the_exported_one() {
    assert_eq!(TfarPlugin::VERSION.to_str().unwrap(), PLUGIN_VERSION);
}

#[test]
fn a_new_plugin_runs_nothing_and_shutdown_is_harmless() {
    let plugin = TfarPlugin::new();
    assert!(!plugin.is_running());
    plugin.shutdown();
    assert!(!plugin.is_running());
}

#[test]
fn the_plugin_serves_the_1_0_flavour() {
    assert_eq!(FLAVOUR, Flavour::V1_0);
}
