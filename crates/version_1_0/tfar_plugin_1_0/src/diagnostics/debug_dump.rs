//! The diagnostics dump of `collectDebugInfo` and `/tfar full`: the reports written as text
//! files under `<config path>/logs/<dd-mm_HH-MM-SS>/`.
//!
//! **Role:** the dump of `reference/tfar_1_0/ts/src/CommandProcessor.cpp:440-457` and
//! `Teamspeak.cpp:857-905`, with the folder from TeamSpeak's `getConfigPath` instead of
//! `%APPDATA%`.
//!
//! **Position:** called from the async message path and the chat command handler.
//!
//! **Signals and state:** reads the runtime through `reports`; writes files only.
//!
//! **Invariants:** a missing config path or an unwritable folder is logged and writes nothing;
//! the folder name is the UTC time, as the plugin carries no time zone database.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::diagnostics::reports::{
    diagnostics_report, game_messages_report, positions_report, settings_report,
};
use crate::runtime::PluginRuntime;

#[cfg(test)]
#[path = "tests/debug_dump.rs"]
mod tests;

/// The folder under the config path that holds every dump.
pub const DUMP_FOLDER: &str = "logs";

/// Seconds in a day.
const SECONDS_PER_DAY: u64 = 86_400;

/// The `dd-mm_HH-MM-SS` name of the moment `unix_seconds` (UTC).
pub fn dump_folder_name(unix_seconds: u64) -> String {
    let days = unix_seconds / SECONDS_PER_DAY;
    let seconds_of_day = unix_seconds % SECONDS_PER_DAY;
    let (_, month, day) = civil_from_days(days);
    format!(
        "{day:02}-{month:02}_{:02}-{:02}-{:02}",
        seconds_of_day / 3600,
        seconds_of_day % 3600 / 60,
        seconds_of_day % 60
    )
}

/// The proleptic Gregorian `(year, month, day)` of `days` since 1970-01-01 (Howard Hinnant's
/// `civil_from_days`).
pub const fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let shifted = days + 719_468;
    let era = shifted / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

/// The dump folder for now under `config_path`.
pub fn dump_folder(config_path: &Path) -> PathBuf {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    config_path
        .join(DUMP_FOLDER)
        .join(dump_folder_name(seconds))
}

/// Writes the dump into a new folder of its own.
pub fn write_debug_dump(runtime: &PluginRuntime) {
    let Some(config_path) = runtime.ts().config_path() else {
        tracing::warn!("no TeamSpeak config path: the diagnostics dump is not written");
        return;
    };
    let folder = dump_folder(&config_path);
    let files = [
        ("diag.log", diagnostics_report(runtime)),
        ("pos.log", positions_report(runtime)),
        ("settings.log", settings_report(runtime)),
        ("game_messages.log", game_messages_report(runtime)),
    ];
    let result = std::fs::create_dir_all(&folder).and_then(|()| {
        files
            .iter()
            .try_for_each(|(name, text)| std::fs::write(folder.join(name), text))
    });
    match result {
        Ok(()) => tracing::info!(folder = %folder.display(), "diagnostics dump written"),
        Err(error) => {
            tracing::warn!(%error, folder = %folder.display(), "the diagnostics dump failed");
        }
    }
}
