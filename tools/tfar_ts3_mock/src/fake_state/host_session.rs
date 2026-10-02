//! The process-global fake state and [`HostSession`], which lets one mock host at a time own it.
//!
//! **Role:** holds the one [`FakeTeamSpeak`] the `extern "C"` fakes reach (a function pointer
//! carries no context, so the state is global) and serialises the hosts that use it, so tests
//! that load plugins in parallel threads take turns.
//!
//! **Position:** the base of `fake_state`; `fake_functions` reads and writes the state through
//! [`with_fake_teamspeak`], and [`MockHost`](crate::MockHost) holds a [`HostSession`] for its
//! lifetime.
//!
//! **Signals and state:** `FAKE_TEAMSPEAK` (a re-entrant `parking_lot` mutex over a `RefCell`,
//! taken briefly by every fake and driver, from any thread), `SESSION_IS_ACTIVE` and its condition variable (taken by
//! [`HostSession::begin`] and its drop).
//!
//! **Invariants:** at most one session is active per process; a session starts from fresh state
//! with its own plugin and config folders, and its end frees every block still handed out; no
//! plugin code runs while `FAKE_TEAMSPEAK` is locked, and a nested use on one thread fails on the
//! `RefCell` rather than deadlocking.

use std::path::PathBuf;

use std::cell::RefCell;

use parking_lot::{Condvar, Mutex, ReentrantMutex};

use super::fake_teamspeak::FakeTeamSpeak;
use crate::temporary_folder::TemporaryFolder;
use crate::{Error, Result};

/// The fake TeamSpeak every fake answers from. The lock is re-entrant and the state a `RefCell`,
/// so a nested use on one thread (a test reading the host inside `with_fake_teamspeak`) fails at
/// once on the `RefCell` instead of deadlocking.
static FAKE_TEAMSPEAK: ReentrantMutex<RefCell<FakeTeamSpeak>> =
    ReentrantMutex::new(RefCell::new(FakeTeamSpeak::empty()));
/// Whether a [`HostSession`] owns the fake state.
static SESSION_IS_ACTIVE: Mutex<bool> = Mutex::new(false);
/// Signalled when a session ends.
static SESSION_ENDED: Condvar = Condvar::new();

/// Runs `change` on the fake state under its lock and returns its answer.
///
/// `change` must not call into the plugin or back into the fake state: the plugin may call a fake,
/// which uses the same state.
pub(crate) fn with_fake_teamspeak<Answer>(
    change: impl FnOnce(&mut FakeTeamSpeak) -> Answer,
) -> Answer {
    let guard = FAKE_TEAMSPEAK.lock();
    let mut fake = guard.borrow_mut();
    change(&mut fake)
}

/// Exclusive use of the fake state, from [`HostSession::begin`] until drop.
#[derive(Debug)]
pub(crate) struct HostSession {
    /// The session's own folder for the default plugin and config paths, when it made one.
    _folder: Option<TemporaryFolder>,
}

impl HostSession {
    /// Waits until no other session is active, then resets the fake state. `plugin_path` and
    /// `config_path` replace the folders the session would create under the system temp folder.
    ///
    /// # Errors
    ///
    /// [`Error::TemporaryFolder`] when a default folder cannot be created.
    pub(crate) fn begin(
        plugin_path: Option<PathBuf>,
        config_path: Option<PathBuf>,
    ) -> Result<Self> {
        let folder = if plugin_path.is_some() && config_path.is_some() {
            None
        } else {
            Some(create_session_folder()?)
        };
        let default_path = |name: &str| {
            folder
                .as_ref()
                .map(|folder| folder.path().join(name))
                .unwrap_or_default()
        };
        let plugin_path = plugin_path.unwrap_or_else(|| default_path("plugins"));
        let config_path = config_path.unwrap_or_else(|| default_path("config"));

        let mut is_active = SESSION_IS_ACTIVE.lock();
        while *is_active {
            SESSION_ENDED.wait(&mut is_active);
        }
        *is_active = true;
        drop(is_active);

        with_fake_teamspeak(|fake| {
            fake.ledger.release_all();
            *fake = FakeTeamSpeak::fresh(plugin_path, config_path);
        });
        Ok(Self { _folder: folder })
    }
}

impl Drop for HostSession {
    fn drop(&mut self) {
        with_fake_teamspeak(|fake| fake.ledger.release_all());
        *SESSION_IS_ACTIVE.lock() = false;
        SESSION_ENDED.notify_one();
    }
}

/// A fresh temp folder holding empty `plugins/` and `config/` folders.
fn create_session_folder() -> Result<TemporaryFolder> {
    let folder =
        TemporaryFolder::create("tfar_ts3_mock").map_err(|source| Error::TemporaryFolder {
            path: std::env::temp_dir(),
            source,
        })?;
    for name in ["plugins", "config"] {
        let path = folder.path().join(name);
        std::fs::create_dir_all(&path).map_err(|source| Error::TemporaryFolder { path, source })?;
    }
    Ok(folder)
}
