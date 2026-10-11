//! Unit tests for `accepts_sender`, against the `strncmp(pluginName, PLUGIN_NAME,
//! strlen(PLUGIN_NAME))` test of `reference/tfar_0_9_12/ts/src/plugin.cpp:3116`.

use super::*;

#[test]
fn every_name_starting_with_the_prefix_is_accepted() {
    // plugin.cpp:53-54 name the Windows builds.
    assert!(accepts_sender("task_force_radio"));
    assert!(accepts_sender("task_force_radio_win32"));
    assert!(accepts_sender("task_force_radio_win64"));
    assert!(accepts_sender("task_force_radio_linux_x64"));
}

#[test]
fn other_names_are_refused() {
    assert!(!accepts_sender(""));
    assert!(!accepts_sender("task_force_radi"));
    assert!(!accepts_sender("Task_Force_Radio_win64"));
    assert!(!accepts_sender("my_task_force_radio"));
    assert!(!accepts_sender("tfar"));
}
