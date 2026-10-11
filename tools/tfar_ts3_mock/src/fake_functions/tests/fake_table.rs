//! Unit tests for the fake table: slot coverage against the original plugins and the SDK
//! header, and the recording stubs.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::*;
use crate::fake_state::host_session::HostSession;

/// The repository root, two folders above this crate.
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_owned()
}

/// Every `ts3Functions.<name>` the original plugin sources under `folder` call.
fn teamspeak_functions_called_under(folder: &Path, names: &mut BTreeSet<String>) {
    for entry in std::fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            teamspeak_functions_called_under(&path, names);
            continue;
        }
        let is_source = path
            .extension()
            .is_some_and(|extension| extension == "cpp" || extension == "hpp" || extension == "h");
        if !is_source {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        let prefix = "ts3Functions.";
        for (at, _) in text.match_indices(prefix) {
            let name: String = text[at + prefix.len()..]
                .chars()
                .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
                .collect();
            if !name.is_empty() {
                names.insert(name);
            }
        }
    }
}

/// The SDK function names in `ts3_functions.h` order, one per `(*name)(` line.
fn sdk_slot_names() -> Vec<String> {
    let header =
        repository_root().join("crates/teamspeak/tfar_ts3_host/sdk_headers/ts3_functions.h");
    std::fs::read_to_string(header)
        .unwrap()
        .lines()
        .filter_map(|line| {
            let start = line.find("(*")? + 2;
            let length = line[start..].find(')')?;
            Some(line[start..start + length].to_owned())
        })
        .collect()
}

/// The address in slot `slot` of `table`.
fn slot_address(table: &Ts3Functions, slot: usize) -> usize {
    assert!(slot < SLOT_COUNT);
    // SAFETY: the table is `SLOT_COUNT` pointer-sized function pointers; `slot` is in bounds.
    unsafe { (&raw const *table).cast::<usize>().add(slot).read() }
}

#[test]
fn every_function_the_tfar_plugins_call_has_a_typed_fake() {
    let mut called = BTreeSet::new();
    for plugin in ["reference/tfar_1_0/ts/src", "reference/tfar_0_9_12/ts/src"] {
        teamspeak_functions_called_under(&repository_root().join(plugin), &mut called);
    }
    let faked: BTreeSet<String> = FAKED_FUNCTIONS
        .iter()
        .map(|function| function.sdk_name.to_owned())
        .collect();
    assert_eq!(
        called.len(),
        32,
        "the originals call 32 functions: {called:?}"
    );
    let missing: Vec<&String> = called.difference(&faked).collect();
    assert!(missing.is_empty(), "no typed fake for {missing:?}");
    assert_eq!(faked, called, "a fake for a function no original calls");
}

#[test]
fn every_faked_slot_index_matches_the_sdk_header_order() {
    let names = sdk_slot_names();
    assert_eq!(names.len(), SLOT_COUNT);
    for function in FAKED_FUNCTIONS {
        assert_eq!(
            names[function.slot], function.sdk_name,
            "slot {} is {} in the header",
            function.slot, names[function.slot]
        );
    }
}

#[test]
fn every_faked_slot_holds_its_fake_and_every_other_slot_its_own_stub() {
    let table = fake_function_table();
    let faked_slots: BTreeSet<usize> = FAKED_FUNCTIONS
        .iter()
        .map(|function| function.slot)
        .collect();
    for slot in 0..SLOT_COUNT {
        let stub = unexpected_slot_stub(slot) as usize;
        if faked_slots.contains(&slot) {
            assert_ne!(
                slot_address(&table, slot),
                stub,
                "slot {slot} still holds its stub"
            );
        } else {
            assert_eq!(
                slot_address(&table, slot),
                stub,
                "slot {slot} holds no stub"
            );
        }
    }
    let distinct_stubs: BTreeSet<usize> = (0..SLOT_COUNT)
        .map(|slot| unexpected_slot_stub(slot) as usize)
        .collect();
    assert_eq!(distinct_stubs.len(), SLOT_COUNT, "two slots share a stub");
}

#[test]
fn an_unexpected_slot_logs_its_index_and_answers_not_implemented() {
    let _session = HostSession::begin(None, None).unwrap();
    let table = fake_function_table();
    let mut version: *mut std::ffi::c_char = std::ptr::null_mut();
    let answer = (table.get_client_lib_version)(&raw mut version);
    assert_eq!(answer, ERROR_NOT_IMPLEMENTED);
    assert!(version.is_null());
    let destroyed = (table.destroy_server_connection_handler)(7);
    assert_eq!(destroyed, ERROR_NOT_IMPLEMENTED);
    let calls = with_fake_teamspeak(|fake| fake.calls().to_vec());
    assert_eq!(
        calls,
        vec![
            HostCall::UnexpectedFunction { slot: 0 },
            HostCall::UnexpectedFunction { slot: 3 },
        ]
    );
}
