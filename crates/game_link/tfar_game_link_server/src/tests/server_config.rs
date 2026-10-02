//! Unit tests for `ServerConfig`.

use super::*;

#[test]
fn a_new_config_has_the_spec_periods() {
    let config = ServerConfig::new(Flavour::V1_0, 0);
    assert_eq!(config.status_interval, Duration::from_millis(500));
    assert_eq!(config.bind_retry_interval, Duration::from_secs(5));
    assert_eq!(config.write_timeout, Duration::from_millis(1000));
    assert_eq!(config.hello_timeout, Duration::from_millis(1000));
}

#[test]
fn needcfg_starts_true_for_1_0_and_false_for_0_9() {
    assert!(ServerConfig::new(Flavour::V1_0, 0).initial_needcfg);
    assert!(!ServerConfig::new(Flavour::V0_9, 0).initial_needcfg);
}

#[test]
fn the_plugin_config_listens_on_the_spec_port() {
    assert_eq!(ServerConfig::for_plugin(Flavour::V0_9, false).port, 47810);
    assert_eq!(ServerConfig::for_plugin(Flavour::V0_9, true).port, 47811);
    assert_eq!(ServerConfig::for_plugin(Flavour::V1_0, false).port, 47820);
    assert_eq!(ServerConfig::for_plugin(Flavour::V1_0, true).port, 47821);
}
