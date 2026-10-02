//! The error type of `tfar_ts3_settings_db`.

use std::path::PathBuf;

/// Every failure `tfar_ts3_settings_db` reports.
///
/// The plugins treat every one of them as "no push-to-talk delay", as the originals do when the
/// query finds nothing.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No file exists at the settings database path.
    #[error("TeamSpeak's settings database {} does not exist", .path.display())]
    MissingDatabase {
        /// The path that was looked up.
        path: PathBuf,
    },
    /// SQLite could not open the file read-only.
    #[error("cannot open TeamSpeak's settings database {} read-only: {source}", .path.display())]
    Open {
        /// The database path.
        path: PathBuf,
        /// SQLite's reason.
        #[source]
        source: rusqlite::Error,
    },
    /// The database has no `Profiles` table.
    #[error("TeamSpeak's settings database {} has no `Profiles` table", .path.display())]
    MissingProfilesTable {
        /// The database path.
        path: PathBuf,
    },
    /// The `Profiles` table has no `Capture/Default/PreProcessing` row.
    #[error(
        "TeamSpeak's settings database {} has no `Capture/Default/PreProcessing` profile",
        .path.display()
    )]
    MissingPreProcessingRow {
        /// The database path.
        path: PathBuf,
    },
    /// A query failed for another reason (a locked or corrupt file, a changed schema).
    #[error("the query on TeamSpeak's settings database {} failed: {source}", .path.display())]
    Query {
        /// The database path.
        path: PathBuf,
        /// SQLite's reason.
        #[source]
        source: rusqlite::Error,
    },
    /// The `delay_ptt_msecs` line holds no whole number of milliseconds.
    #[error("the pre-processing line `{line}` holds no whole number of milliseconds")]
    InvalidMilliseconds {
        /// The line as stored, without its line end.
        line: String,
    },
}

/// The result of a fallible `tfar_ts3_settings_db` operation.
pub type Result<T> = core::result::Result<T, Error>;
