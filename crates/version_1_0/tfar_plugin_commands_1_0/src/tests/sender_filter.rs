//! Unit tests for `accepts_sender`, against `strncmp(pluginName, "TFAR", 4)`
//! (`reference/tfar_1_0/ts/src/plugin.cpp:1079`).

use super::*;

#[test]
fn a_plugin_name_starting_with_tfar_is_accepted() {
    assert!(accepts_sender("TFAR"));
    assert!(accepts_sender("TFAR_task_force_radio"));
    assert!(accepts_sender(b"TFAR\xFF".as_slice()));
}

#[test]
fn any_other_name_is_refused() {
    for name in ["tfar", "TFA", "", "task_force_radio", " TFAR"] {
        assert!(!accepts_sender(name), "{name:?}");
    }
}
