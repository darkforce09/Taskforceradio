//! The error type of `tfar_audio::hrtf`.

use std::path::PathBuf;

/// Every failure `tfar_audio::hrtf` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The sphere file could not be read (missing, unreadable, a folder).
    #[error("cannot read the HRIR sphere {path:?}: {source}")]
    SphereFileUnreadable {
        /// The path that was asked for.
        path: PathBuf,
        /// The error the file system gave.
        source: std::io::Error,
    },
    /// The sphere file was read but is not a well-formed HRIR sphere; it is refused before the
    /// `hrtf` crate sees it, because that crate would abort the process on some malformed input.
    #[error("the HRIR sphere {path:?} is malformed: {reason}")]
    SphereFileMalformed {
        /// The path of the file.
        path: PathBuf,
        /// What is wrong with it.
        reason: String,
    },
    /// The `hrtf` crate refused the sphere after the structural check passed.
    #[error("the hrtf crate refused the HRIR sphere {path:?}: {detail}")]
    SphereRejected {
        /// The path of the file.
        path: PathBuf,
        /// The `hrtf` crate's error, as its debug text (it implements no `Display`).
        detail: String,
    },
}

/// The result of a fallible `tfar_audio::hrtf` operation.
pub type Result<T> = core::result::Result<T, Error>;
