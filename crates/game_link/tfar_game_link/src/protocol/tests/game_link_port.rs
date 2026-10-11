//! Unit tests for `port`.

use super::*;

#[test]
fn each_flavour_has_its_normal_and_debug_port_from_the_spec() {
    assert_eq!(port(Flavour::V0_9, false), 47810);
    assert_eq!(port(Flavour::V0_9, true), 47811);
    assert_eq!(port(Flavour::V1_0, false), 47820);
    assert_eq!(port(Flavour::V1_0, true), 47821);
}
