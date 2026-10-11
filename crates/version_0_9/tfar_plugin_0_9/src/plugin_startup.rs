//! What `ts3plugin_init` gathers before the plugin runs: the plugin folder, the HRIR sphere and
//! TeamSpeak's push-to-talk delay.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tfar_audio::hrtf::HrirLibrary;
use tfar_ts3_host::Ts3;
use tfar_ts3_settings_db::{read_push_to_talk_delay, settings_database_path};

use crate::plugin_core::PluginResources;
use crate::radio_sounds::hrir_sphere_path;
use crate::team_speak::TeamSpeakActions;

#[cfg(test)]
#[path = "tests/plugin_startup.rs"]
mod tests;

/// The chat line that says the HRIR sphere is missing, naming the path looked at.
pub fn missing_sphere_message(path: &Path) -> String {
    format!(
        "[B]Task Force Radio[/B]: the HRIR sphere {} could not be loaded; voices are placed by \
         simple stereo panning instead of HRTF.",
        path.display()
    )
}

/// Loads the HRIR sphere under `plugin_folder`; without it, says so in the chat tab through
/// `team_speak` and returns `None`, which turns the HRTF off.
pub fn load_hrir_sphere(
    plugin_folder: &Path,
    team_speak: &dyn TeamSpeakActions,
) -> Option<Arc<HrirLibrary>> {
    let path = hrir_sphere_path(plugin_folder);
    match HrirLibrary::load(&path) {
        Ok(library) => Some(library),
        Err(error) => {
            tracing::warn!(%error, "the HRIR sphere could not be loaded; the HRTF is off");
            team_speak.print_message(&missing_sphere_message(&path));
            None
        }
    }
}

/// TeamSpeak's push-to-talk delay from `settings.db` under `config_folder`, read once; zero
/// when the database or the setting is missing.
pub fn push_to_talk_delay(config_folder: Option<&Path>) -> Duration {
    let Some(config_folder) = config_folder else {
        return Duration::ZERO;
    };
    match read_push_to_talk_delay(&settings_database_path(config_folder)) {
        Ok(delay) => delay.effective(),
        Err(error) => {
            tracing::debug!(%error, "no push-to-talk delay was read; releases are immediate");
            Duration::ZERO
        }
    }
}

/// The resources of a plugin started by TeamSpeak through `ts3`. The plugin folder is the one
/// `getPluginPath` names for the current plugin ID, or else the folder of the loaded `.so`
/// ([`Ts3::plugin_folder`]).
pub fn gather_resources(ts3: &Ts3, team_speak: &dyn TeamSpeakActions) -> PluginResources {
    let plugin_folder = ts3.plugin_folder().unwrap_or_else(|error| {
        tracing::warn!(%error, "TeamSpeak named no plugin folder; the radio sounds are off");
        PathBuf::new()
    });
    let config_folder = ts3
        .config_path()
        .map_err(|error| tracing::debug!(%error, "TeamSpeak named no config folder"))
        .ok();
    PluginResources {
        hrir: load_hrir_sphere(&plugin_folder, team_speak),
        push_to_talk_delay: push_to_talk_delay(config_folder.as_deref()),
        plugin_folder,
    }
}
