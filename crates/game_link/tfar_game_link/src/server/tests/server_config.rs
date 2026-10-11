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

#[test]
fn a_one_port_config_listens_on_that_port_alone() {
    let config = ServerConfig::new(Flavour::V0_9, 50_123);
    assert_eq!(config.debug_port, None);
    assert_eq!(config.listen_ports().collect::<Vec<_>>(), [50_123]);
    assert_eq!(
        ServerConfig::for_plugin(Flavour::V1_0, true)
            .listen_ports()
            .collect::<Vec<_>>(),
        [47821]
    );
}

#[test]
fn the_plugin_config_on_both_ports_lists_the_normal_port_first() {
    let config = ServerConfig::for_plugin_on_both_ports(Flavour::V0_9);
    assert_eq!(config.port, 47810);
    assert_eq!(config.debug_port, Some(47811));
    assert_eq!(config.listen_ports().collect::<Vec<_>>(), [47810, 47811]);
    assert_eq!(
        ServerConfig::for_plugin_on_both_ports(Flavour::V1_0)
            .listen_ports()
            .collect::<Vec<_>>(),
        [47820, 47821]
    );
}

#[test]
fn with_debug_port_keeps_every_other_setting() {
    let base = ServerConfig {
        status_interval: Duration::from_millis(7),
        ..ServerConfig::new(Flavour::V1_0, 0)
    };
    let both = base.with_debug_port(0);
    assert_eq!(both.debug_port, Some(0));
    assert_eq!(
        ServerConfig {
            debug_port: None,
            ..both
        },
        base
    );
}
