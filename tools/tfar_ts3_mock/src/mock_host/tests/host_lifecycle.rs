//! Unit tests for starting, querying and shutting down a plugin, on the in-process plugin.

use std::path::Path;

use super::*;
use crate::mock_host::in_process_plugin::{
    PluginEvent, freed_info_blocks, in_process_exports, old_api_version, start_in_process_host,
    take_events, try_start, try_start_with,
};

#[test]
fn the_plugin_id_constants_agree() {
    assert_eq!(MOCK_PLUGIN_ID_C_STRING.to_str(), Ok(MOCK_PLUGIN_ID));
}

#[test]
fn start_sets_the_table_then_registers_the_id_then_runs_init() {
    let host = start_in_process_host();
    let events = take_events();
    let config_path = host.config_path();
    assert_eq!(
        events,
        vec![
            PluginEvent::RegisteredPluginId("mock_plugin_id".to_owned()),
            PluginEvent::Init {
                config_path: format!("{}/", config_path.display()),
            },
        ]
    );
    assert_eq!(
        host.calls(),
        vec![
            HostCall::Query(crate::HostQuery::ConfigPath),
            HostCall::LogMessage {
                text: "init".to_owned(),
                level: tfar_ts3_host::LogLevel::Info,
                channel: "test_plugin".to_owned(),
                server: ServerConnectionHandlerId(0),
            },
        ]
    );
    assert!(config_path.is_dir());
    assert!(host.plugin_path().is_dir());
}

#[test]
fn the_metadata_is_read_from_the_exports() {
    let host = start_in_process_host();
    assert_eq!(
        host.metadata(),
        &PluginMetadata {
            name: "In-process Test Plugin".to_owned(),
            version: "1.2.3".to_owned(),
            api_version: 26,
            author: "Test Author".to_owned(),
            description: "Records what the mock does".to_owned(),
            requests_autoload: true,
            offers_configure: None,
            command_keyword: Some("test".to_owned()),
            info_title: None,
        }
    );
    assert_eq!(host.plugin().path(), Path::new("in_process_plugin"));
}

#[test]
fn a_failing_init_is_an_error() {
    let (_users, started) = try_start(&in_process_exports(), 1);
    assert!(matches!(started, Err(Error::InitFailed { code: 1 })));
}

#[test]
fn a_plugin_for_another_api_is_refused_before_it_runs() {
    let mut exports = in_process_exports();
    exports.api_version = old_api_version;
    let (_users, started) = try_start(&exports, 0);
    assert!(matches!(
        started,
        Err(Error::ApiVersionMismatch {
            expected: 26,
            found: 25
        })
    ));
    assert_eq!(take_events(), Vec::new());
}

#[test]
fn shutdown_runs_once_also_when_the_host_is_dropped() {
    let mut host = start_in_process_host();
    take_events();
    host.shutdown();
    host.shutdown();
    assert_eq!(take_events(), vec![PluginEvent::Shutdown]);
    drop(host);
    assert_eq!(take_events(), Vec::new());

    let host = start_in_process_host();
    take_events();
    drop(host);
    assert_eq!(take_events(), vec![PluginEvent::Shutdown]);
}

#[test]
fn info_data_copies_the_text_and_hands_the_block_back() {
    let host = start_in_process_host();
    assert_eq!(
        host.info_data(7, InfoKind::Client).unwrap(),
        Some("info 2 7".to_owned())
    );
    assert_eq!(
        host.info_data(3, InfoKind::Channel).unwrap(),
        Some("info 1 3".to_owned())
    );
    assert_eq!(freed_info_blocks(), 2);
    assert!(host.leak_report().is_clean());
}

#[test]
fn an_info_block_with_no_free_memory_export_is_a_plugin_leak() {
    let mut exports = in_process_exports();
    exports.free_memory = None;
    let (_users, host) = try_start(&exports, 0);
    let host = host.unwrap();
    assert_eq!(
        host.info_data(1, InfoKind::Server).unwrap(),
        Some("info 0 1".to_owned())
    );
    let leaks = host.leak_report();
    assert_eq!(leaks.unfreed_plugin_blocks, 1);
    assert!(!leaks.is_clean());
}

#[test]
fn process_command_answers_handled_or_not() {
    let host = start_in_process_host();
    host.take_calls();
    assert_eq!(
        host.process_command("ping").unwrap(),
        CommandOutcome::Handled
    );
    assert_eq!(
        host.process_command("other").unwrap(),
        CommandOutcome::NotHandled
    );
    assert_eq!(
        host.take_calls(),
        vec![HostCall::PrintMessageToCurrentTab {
            text: "pong".to_owned()
        }]
    );
    assert!(matches!(
        host.process_command("a\0b"),
        Err(Error::InteriorNul { .. })
    ));
}

#[test]
fn a_driver_whose_export_is_missing_is_an_error() {
    let mut exports = in_process_exports();
    exports.info_data = None;
    exports.process_command = None;
    let (_users, host) = try_start(&exports, 0);
    let host = host.unwrap();
    assert!(matches!(
        host.info_data(1, InfoKind::Server),
        Err(Error::MissingExport {
            symbol: "ts3plugin_infoData"
        })
    ));
    assert!(matches!(
        host.process_command("ping"),
        Err(Error::MissingExport {
            symbol: "ts3plugin_processCommand"
        })
    ));
}

#[test]
fn the_fake_state_can_be_read_and_set_up_through_the_host() {
    let host = start_in_process_host();
    host.with_fake_teamspeak(|fake| {
        fake.preprocessor_values
            .insert("vad".to_owned(), "true".to_owned());
    });
    let vad = host.with_fake_teamspeak(|fake| fake.preprocessor_values.get("vad").cloned());
    assert_eq!(vad.as_deref(), Some("true"));
    assert_eq!(host.current_server(), ServerConnectionHandlerId(0));
}

#[test]
fn a_missing_file_is_a_load_error() {
    let error = MockHost::load(Path::new("/nonexistent/libtfar_plugin.so")).unwrap_err();
    assert!(matches!(error, Error::LoadLibrary { .. }), "{error}");
}

#[test]
#[cfg(target_os = "linux")]
fn a_library_without_the_plugin_exports_is_refused() {
    let error = PluginLibrary::open(Path::new("libc.so.6")).unwrap_err();
    assert!(
        matches!(
            error,
            Error::MissingRequiredSymbol {
                symbol: "ts3plugin_name",
                ..
            }
        ),
        "{error}"
    );
}

#[test]
fn hosts_given_folders_answer_with_them() {
    let folder = crate::temporary_folder::TemporaryFolder::create("tfar_ts3_mock-options").unwrap();
    let options = HostOptions {
        plugin_path: Some(folder.path().join("plugins")),
        config_path: Some(folder.path().join("config")),
    };
    let (_users, host) = try_start_with(&in_process_exports(), 0, &options);
    let host = host.unwrap();
    assert_eq!(host.plugin_path(), folder.path().join("plugins"));
    assert_eq!(host.config_path(), folder.path().join("config"));
    assert_eq!(
        take_events()[1],
        PluginEvent::Init {
            config_path: format!("{}/", folder.path().join("config").display()),
        }
    );
}

#[test]
#[should_panic(expected = "already borrowed")]
fn a_nested_use_of_the_fake_state_fails_at_once_instead_of_deadlocking() {
    let host = start_in_process_host();
    host.with_fake_teamspeak(|_| host.current_server());
}
