//! Unit tests for [`Ts3::plugin_folder`]: TeamSpeak's folder first, the loaded library's folder
//! when TeamSpeak names none or the ID is not registered yet.

use std::ffi::c_char;
use std::path::{Path, PathBuf};

use super::{PluginFolderSource, loaded_library_folder};
use crate::Ts3;
use crate::fake_ts3_table::fake_table;

extern "C" fn fake_teamspeak_plugin_path(
    path: *mut c_char,
    _max_length: usize,
    _id: *const c_char,
) {
    let text = b"/home/user/.ts3client/plugins\0";
    // SAFETY: the code under test passes a writable buffer of 4096 bytes, far longer.
    unsafe { std::ptr::copy_nonoverlapping(text.as_ptr().cast::<c_char>(), path, text.len()) };
}

/// What TeamSpeak does for a plugin ID it does not know: writes nothing into the zeroed buffer.
extern "C" fn fake_empty_plugin_path(_path: *mut c_char, _max_length: usize, _id: *const c_char) {}

fn ts3_with_plugin_path(get_plugin_path: extern "C" fn(*mut c_char, usize, *const c_char)) -> Ts3 {
    let mut table = fake_table();
    table.get_plugin_path = get_plugin_path;
    Ts3::from_functions(table)
}

/// The folder of the test executable, which holds `loaded_library_folder` in this crate's tests.
fn test_executable_folder() -> PathBuf {
    let executable = std::fs::canonicalize(std::env::current_exe().unwrap()).unwrap();
    executable.parent().unwrap().to_path_buf()
}

#[test]
fn the_folder_teamspeak_names_comes_first() {
    let ts3 = ts3_with_plugin_path(fake_teamspeak_plugin_path);
    ts3.register_plugin_id(c"tfar_id");
    assert_eq!(
        ts3.plugin_folder_with_source().unwrap(),
        (
            PathBuf::from("/home/user/.ts3client/plugins"),
            PluginFolderSource::TeamSpeak
        )
    );
    assert_eq!(
        ts3.plugin_folder().unwrap(),
        Path::new("/home/user/.ts3client/plugins")
    );
}

#[test]
fn an_empty_plugin_path_falls_back_to_the_folder_of_the_loaded_library() {
    let ts3 = ts3_with_plugin_path(fake_empty_plugin_path);
    ts3.register_plugin_id(c"stale_id");
    assert_eq!(
        ts3.plugin_folder_with_source().unwrap(),
        (test_executable_folder(), PluginFolderSource::LoadedLibrary)
    );
}

#[test]
fn without_a_registered_id_the_folder_of_the_loaded_library_answers() {
    let ts3 = ts3_with_plugin_path(fake_teamspeak_plugin_path);
    assert_eq!(
        ts3.plugin_folder_with_source().unwrap(),
        (test_executable_folder(), PluginFolderSource::LoadedLibrary)
    );
}

#[test]
fn the_loaded_library_folder_is_an_existing_absolute_folder() {
    let folder = loaded_library_folder().unwrap();
    assert!(folder.is_absolute(), "{}", folder.display());
    assert!(folder.is_dir(), "{}", folder.display());
}
