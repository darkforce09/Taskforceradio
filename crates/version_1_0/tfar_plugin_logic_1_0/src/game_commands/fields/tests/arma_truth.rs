//! Unit tests for `is_true`, against `helpers::isTrue` (`reference/tfar_1_0/ts/src/helpers.cpp:296-300`).

use super::*;

#[test]
fn only_true_and_one_are_true() {
    assert!(is_true("true"));
    assert!(is_true("1"));
    for text in ["false", "0", "TRUE", "True", "", "11", "yes", " true"] {
        assert!(!is_true(text), "{text:?}");
    }
}
