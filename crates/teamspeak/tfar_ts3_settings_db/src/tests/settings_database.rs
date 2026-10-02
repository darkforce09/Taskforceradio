//! Unit tests for the read-only query on temporary `settings.db` files.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use rusqlite::Connection;

use super::*;

/// A fresh folder under the system temp folder, removed when dropped.
struct TemporaryFolder(PathBuf);

impl TemporaryFolder {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let unique = format!(
            "tfar_ts3_settings_db-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn database(&self) -> PathBuf {
        settings_database_path(&self.0)
    }
}

impl Drop for TemporaryFolder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Writes a `Profiles` table at `path` with one `(key, value)` row per entry.
fn write_profiles(path: &Path, rows: &[(&str, &str)]) {
    let connection = Connection::open(path).unwrap();
    connection
        .execute("CREATE TABLE Profiles (key TEXT, value TEXT)", [])
        .unwrap();
    for (key, value) in rows {
        connection
            .execute(
                "INSERT INTO Profiles (key, value) VALUES (?1, ?2)",
                [key, value],
            )
            .unwrap();
    }
}

const KEY: &str = "Capture/Default/PreProcessing";

#[test]
fn the_database_path_is_settings_db_in_the_config_folder() {
    assert_eq!(
        settings_database_path(Path::new("/config/")),
        PathBuf::from("/config/settings.db")
    );
}

#[test]
fn an_enabled_delay_with_milliseconds_is_read_from_the_database() {
    let folder = TemporaryFolder::new();
    write_profiles(
        &folder.database(),
        &[
            ("Capture/Other/PreProcessing", "delay_ptt=false"),
            (KEY, "vad=false\ndelay_ptt=true\ndelay_ptt_msecs=450\n"),
        ],
    );
    let delay = read_push_to_talk_delay(&folder.database()).unwrap();
    assert_eq!(
        delay,
        PushToTalkDelay {
            enabled: true,
            milliseconds: 450
        }
    );
    assert_eq!(delay.effective(), Duration::from_millis(450));
}

#[test]
fn a_disabled_delay_is_zero_in_effect() {
    let folder = TemporaryFolder::new();
    write_profiles(
        &folder.database(),
        &[(KEY, "delay_ptt=false\ndelay_ptt_msecs=450")],
    );
    let delay = read_push_to_talk_delay(&folder.database()).unwrap();
    assert!(!delay.enabled);
    assert_eq!(delay.effective(), Duration::ZERO);
}

#[test]
fn crlf_lines_in_the_database_are_read() {
    let folder = TemporaryFolder::new();
    write_profiles(
        &folder.database(),
        &[(KEY, "delay_ptt=true\r\ndelay_ptt_msecs=120\r\n")],
    );
    let delay = read_push_to_talk_delay(&folder.database()).unwrap();
    assert_eq!(delay.effective(), Duration::from_millis(120));
}

#[test]
fn a_missing_row_is_a_typed_error() {
    let folder = TemporaryFolder::new();
    write_profiles(
        &folder.database(),
        &[("Capture/Other/PreProcessing", "delay_ptt=true")],
    );
    let error = read_push_to_talk_delay(&folder.database()).unwrap_err();
    assert!(
        matches!(error, Error::MissingPreProcessingRow { .. }),
        "{error:?}"
    );
}

#[test]
fn a_missing_table_is_a_typed_error() {
    let folder = TemporaryFolder::new();
    Connection::open(folder.database())
        .unwrap()
        .execute("CREATE TABLE Other (key TEXT)", [])
        .unwrap();
    let error = read_push_to_talk_delay(&folder.database()).unwrap_err();
    assert!(
        matches!(error, Error::MissingProfilesTable { .. }),
        "{error:?}"
    );
}

#[test]
fn a_missing_file_is_a_typed_error_and_is_not_created() {
    let folder = TemporaryFolder::new();
    let error = read_push_to_talk_delay(&folder.database()).unwrap_err();
    assert!(matches!(error, Error::MissingDatabase { .. }), "{error:?}");
    assert!(!folder.database().exists());
}

#[test]
fn a_file_that_is_not_a_database_is_a_typed_error() {
    let folder = TemporaryFolder::new();
    std::fs::write(folder.database(), b"this is not an SQLite file, just text").unwrap();
    let error = read_push_to_talk_delay(&folder.database()).unwrap_err();
    assert!(
        matches!(error, Error::Open { .. } | Error::Query { .. }),
        "{error:?}"
    );
}

#[test]
fn garbage_milliseconds_in_the_database_are_a_typed_error() {
    let folder = TemporaryFolder::new();
    write_profiles(
        &folder.database(),
        &[(KEY, "delay_ptt=true\ndelay_ptt_msecs=\u{fffd}x")],
    );
    let error = read_push_to_talk_delay(&folder.database()).unwrap_err();
    assert!(
        matches!(error, Error::InvalidMilliseconds { .. }),
        "{error:?}"
    );
}

#[test]
fn a_garbage_profile_value_is_a_disabled_zero_delay() {
    let folder = TemporaryFolder::new();
    write_profiles(&folder.database(), &[(KEY, "\u{0}\u{7}garbage;;==")]);
    let delay = read_push_to_talk_delay(&folder.database()).unwrap();
    assert_eq!(delay, PushToTalkDelay::default());
}

#[test]
fn a_null_profile_value_is_a_disabled_zero_delay() {
    let folder = TemporaryFolder::new();
    let connection = Connection::open(folder.database()).unwrap();
    connection
        .execute("CREATE TABLE Profiles (key TEXT, value TEXT)", [])
        .unwrap();
    connection
        .execute("INSERT INTO Profiles (key, value) VALUES (?1, NULL)", [KEY])
        .unwrap();
    drop(connection);
    let delay = read_push_to_talk_delay(&folder.database()).unwrap();
    assert_eq!(delay, PushToTalkDelay::default());
}

#[test]
fn reading_leaves_the_database_bytes_unchanged() {
    let folder = TemporaryFolder::new();
    write_profiles(&folder.database(), &[(KEY, "delay_ptt=true")]);
    let before = std::fs::read(folder.database()).unwrap();
    read_push_to_talk_delay(&folder.database()).unwrap();
    assert_eq!(std::fs::read(folder.database()).unwrap(), before);
}

#[test]
fn a_database_another_connection_holds_open_is_still_read() {
    let folder = TemporaryFolder::new();
    write_profiles(
        &folder.database(),
        &[(KEY, "delay_ptt=true\ndelay_ptt_msecs=80")],
    );
    let teamspeak = Connection::open(folder.database()).unwrap();
    let delay = read_push_to_talk_delay(&folder.database()).unwrap();
    assert_eq!(delay.effective(), Duration::from_millis(80));
    drop(teamspeak);
}
