//! The read-only query on TeamSpeak's `settings.db` that yields the [`PushToTalkDelay`].
//!
//! **Role:** opens the SQLite file read-only, checks the `Profiles` table is there, reads the
//! `Capture/Default/PreProcessing` value and hands its text to the parser.
//!
//! **Position:** the crate's entry point; the plugin apps call it once from `ts3plugin_init` with
//! the path under TeamSpeak's config folder.
//!
//! **Signals and state:** one connection per call, closed before it returns; runs on whichever
//! thread calls it (TeamSpeak's main thread in the plugins).
//!
//! **Invariants:** the file is opened with `SQLITE_OPEN_READ_ONLY`, so TeamSpeak's open database is
//! never written and a missing file is never created; the query is the one of
//! `reference/tfar_1_0/ts/src/plugin.cpp:351` and `reference/tfar_0_9_12/ts/src/plugin.cpp:2254`.

use std::path::{Path, PathBuf};

use rusqlite::types::ValueRef;
use rusqlite::{Connection, OpenFlags, OptionalExtension};

use crate::{Error, PushToTalkDelay, Result};

/// The file name of TeamSpeak's settings database inside its config folder.
pub const SETTINGS_DATABASE_FILE_NAME: &str = "settings.db";

/// The key of the capture pre-processing profile in the `Profiles` table.
const PRE_PROCESSING_KEY: &str = "Capture/Default/PreProcessing";

#[cfg(test)]
#[path = "tests/settings_database.rs"]
mod tests;

/// The settings database inside TeamSpeak's config folder `config_path`.
pub fn settings_database_path(config_path: &Path) -> PathBuf {
    config_path.join(SETTINGS_DATABASE_FILE_NAME)
}

/// Reads the push-to-talk delay from the `settings.db` file at `settings_db`.
///
/// # Errors
///
/// [`Error::MissingDatabase`] when no file is there; [`Error::Open`] when SQLite cannot open it;
/// [`Error::MissingProfilesTable`] and [`Error::MissingPreProcessingRow`] when the table or the
/// row is absent; [`Error::Query`] when a query fails; [`Error::InvalidMilliseconds`] when the
/// stored length is not a whole number. The plugins treat each as no delay.
pub fn read_push_to_talk_delay(settings_db: &Path) -> Result<PushToTalkDelay> {
    if !settings_db.is_file() {
        return Err(Error::MissingDatabase {
            path: settings_db.to_owned(),
        });
    }
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let connection =
        Connection::open_with_flags(settings_db, flags).map_err(|source| Error::Open {
            path: settings_db.to_owned(),
            source,
        })?;
    let query_error = |source| Error::Query {
        path: settings_db.to_owned(),
        source,
    };
    let has_profiles_table = connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'Profiles'",
            [],
            |_| Ok(()),
        )
        .optional()
        .map_err(query_error)?
        .is_some();
    if !has_profiles_table {
        return Err(Error::MissingProfilesTable {
            path: settings_db.to_owned(),
        });
    }
    let profile = connection
        .query_row(
            "SELECT value FROM Profiles WHERE key = ?1",
            [PRE_PROCESSING_KEY],
            |row| row.get_ref(0).map(profile_text),
        )
        .optional()
        .map_err(query_error)?
        .ok_or_else(|| Error::MissingPreProcessingRow {
            path: settings_db.to_owned(),
        })?;
    PushToTalkDelay::parse_pre_processing(&profile)
}

/// The stored value as text: TEXT and BLOB lossily as UTF-8, numbers as written, NULL as empty
/// (the originals skip a NULL value, which leaves the delay off).
fn profile_text(value: ValueRef<'_>) -> String {
    match value {
        ValueRef::Null => String::new(),
        ValueRef::Integer(number) => number.to_string(),
        ValueRef::Real(number) => number.to_string(),
        ValueRef::Text(bytes) | ValueRef::Blob(bytes) => {
            String::from_utf8_lossy(bytes).into_owned()
        }
    }
}
