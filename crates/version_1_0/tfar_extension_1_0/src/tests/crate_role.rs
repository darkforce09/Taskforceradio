//! Unit tests for `CRATE_ROLE` in `tfar_extension_1_0`.

use super::*;

#[test]
fn crate_role_names_the_crate_it_describes() {
    assert!(CRATE_ROLE.starts_with("tfar_extension_1_0: "));
}
