//! Unit tests for `CRATE_ROLE` in `tfar_radio_effects_0_9`.

use super::*;

#[test]
fn crate_role_names_the_crate_it_describes() {
    assert!(CRATE_ROLE.starts_with("tfar_radio_effects_0_9: "));
}
