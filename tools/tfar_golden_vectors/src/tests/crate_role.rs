//! Unit tests for `CRATE_ROLE` in `tfar_golden_vectors`.

use super::*;

#[test]
fn crate_role_names_the_crate_it_describes() {
    assert!(CRATE_ROLE.starts_with("tfar_golden_vectors: "));
}
