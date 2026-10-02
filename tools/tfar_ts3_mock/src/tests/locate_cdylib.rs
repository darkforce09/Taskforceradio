//! Unit tests for `locate_cdylib` on a fake target tree.

use std::path::PathBuf;

use super::*;
use crate::temporary_folder::TemporaryFolder;

/// A fake `target-x/debug/deps/` tree with a test executable in `deps/`.
fn target_tree() -> (TemporaryFolder, PathBuf) {
    let folder = TemporaryFolder::create("tfar_ts3_mock-locate").unwrap();
    let deps = folder.path().join("target-x/debug/deps");
    std::fs::create_dir_all(&deps).unwrap();
    let executable = deps.join("plugin_scenarios-0123abcd");
    std::fs::write(&executable, b"").unwrap();
    (folder, executable)
}

#[test]
#[cfg(target_os = "linux")]
fn the_file_name_is_the_platform_cdylib_name_with_underscores() {
    assert_eq!(cdylib_file_name("tfar-plugin_1_0"), "libtfar_plugin_1_0.so");
}

#[test]
fn a_library_in_deps_beside_the_test_executable_is_found() {
    let (_folder, executable) = target_tree();
    let library = executable.with_file_name("libtfar_plugin_1_0.so");
    std::fs::write(&library, b"").unwrap();
    assert_eq!(
        locate_cdylib_from(&executable, "tfar_plugin_1_0"),
        Some(library)
    );
}

#[test]
fn a_library_uplifted_to_the_profile_folder_is_found() {
    let (_folder, executable) = target_tree();
    let debug = executable.parent().unwrap().parent().unwrap();
    let library = debug.join("libtfar_plugin_0_9.so");
    std::fs::write(&library, b"").unwrap();
    assert_eq!(
        locate_cdylib_from(&executable, "tfar_plugin_0_9"),
        Some(library)
    );
}

#[test]
fn a_binary_in_the_profile_folder_finds_the_library_in_deps() {
    let (_folder, executable) = target_tree();
    let debug = executable.parent().unwrap().parent().unwrap().to_owned();
    let binary = debug.join("tfar_ts3_mock");
    std::fs::write(&binary, b"").unwrap();
    let library = debug.join("deps/libtfar_plugin_1_0.so");
    std::fs::write(&library, b"").unwrap();
    assert_eq!(
        locate_cdylib_from(&binary, "tfar_plugin_1_0"),
        Some(library)
    );
}

#[test]
fn a_library_that_was_not_built_is_none() {
    let (_folder, executable) = target_tree();
    assert_eq!(locate_cdylib_from(&executable, "tfar_plugin_1_0"), None);
}

#[test]
fn the_running_test_executable_does_not_find_a_crate_that_builds_no_cdylib() {
    assert_eq!(locate_cdylib("tfar_no_such_plugin"), None);
}
