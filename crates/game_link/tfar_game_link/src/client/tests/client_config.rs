//! Unit tests for `ClientConfig`.

use super::*;

#[test]
fn new_uses_the_timeouts_of_the_game_link_spec() {
    let config = ClientConfig::new(Flavour::V1_0, 47820);
    assert_eq!(config.flavour, Flavour::V1_0);
    assert_eq!(config.port, 47820);
    assert_eq!(config.connect_timeout, Duration::from_millis(50));
    assert_eq!(config.hello_timeout, Duration::from_millis(250));
    assert_eq!(config.backoff, Duration::from_secs(1));
    assert_eq!(config.sync_timeout, Duration::from_millis(1000));
    assert_eq!(config.session_dead_after, Duration::from_millis(1000));
    assert_eq!(config.write_timeout, Duration::from_millis(1000));
}
