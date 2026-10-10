//! Where the radio sounds live under the plugin folder, by radio kind and moment.
//!
//! **Role:** name the WAV files of `radio-sounds/` the 0.9 plugin plays: `on` and `off` at game
//! start and end, `local_start` and `local_end` for the local push-to-talk, `remote_start` and
//! `remote_end` for another player's, each in the folder of the radio subtype (`sw`, `lr`, `dd`,
//! `ab`; a phone or an unknown subtype has none).
//!
//! **Position:** used by the game message handling (local sounds through `playWaveFile`), the
//! incoming plugin commands (remote sounds through the playback mixer) and `init` (the preload).
//!
//! **Signals and state:** none; pure path building.
//!
//! **Invariants:** the folder mapping is the original's (`reference/tfar_0_9_12/ts/src/plugin.cpp`
//! 1840-1866 and 3025-3044): `digital` → `sw`, `digital_lr` → `lr`, `dd` → `dd`, `airborne` →
//! `ab`; every path ends in `.wav`.

use std::path::{Path, PathBuf};

use tfar_arma_values::RadioSubtype;

#[cfg(test)]
#[path = "tests/radio_sounds.rs"]
mod tests;

/// The folder of the radio sounds inside TeamSpeak's plugins folder.
pub const RADIO_SOUNDS_FOLDER: &str = "radio-sounds";

/// The folder of the HRIR sphere inside TeamSpeak's plugins folder, beside `radio-sounds/`.
pub const HRIR_FOLDER: &str = "tfar_hrir";

/// The HRIR sphere file inside [`HRIR_FOLDER`].
pub const HRIR_FILE_NAME: &str = "IRC_1002_C.bin";

/// The radio kinds with a sound folder, as the folders are named.
pub const SOUND_FOLDERS: [&str; 4] = ["sw", "lr", "dd", "ab"];

/// The moment of a push-to-talk a sound marks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TangentMoment {
    /// The local player pressed.
    LocalStart,
    /// The local player released.
    LocalEnd,
    /// Another player pressed.
    RemoteStart,
    /// Another player released.
    RemoteEnd,
}

impl TangentMoment {
    /// The file stem of the moment's sound.
    pub const fn file_stem(self) -> &'static str {
        match self {
            Self::LocalStart => "local_start",
            Self::LocalEnd => "local_end",
            Self::RemoteStart => "remote_start",
            Self::RemoteEnd => "remote_end",
        }
    }

    /// Every moment, local first.
    pub const ALL: [Self; 4] = [
        Self::LocalStart,
        Self::LocalEnd,
        Self::RemoteStart,
        Self::RemoteEnd,
    ];
}

/// The sound folder of `subtype`; `None` for a phone or an unknown subtype, which play nothing.
pub fn sound_folder(subtype: &RadioSubtype) -> Option<&'static str> {
    match subtype {
        RadioSubtype::Digital => Some("sw"),
        RadioSubtype::DigitalLongRange => Some("lr"),
        RadioSubtype::DiverDigital => Some("dd"),
        RadioSubtype::Airborne => Some("ab"),
        RadioSubtype::Phone | RadioSubtype::Unknown(_) => None,
    }
}

/// The radio sound files under one plugin folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioSounds {
    root: PathBuf,
}

impl RadioSounds {
    /// The sounds under `plugin_folder` (TeamSpeak's plugins folder).
    pub fn new(plugin_folder: &Path) -> Self {
        Self {
            root: plugin_folder.join(RADIO_SOUNDS_FOLDER),
        }
    }

    /// The game start sound, `radio-sounds/on.wav`.
    pub fn game_start(&self) -> PathBuf {
        self.root.join("on.wav")
    }

    /// The game end sound, `radio-sounds/off.wav`.
    pub fn game_end(&self) -> PathBuf {
        self.root.join("off.wav")
    }

    /// The sound of `moment` for a radio of `subtype`; `None` when the subtype has no folder.
    pub fn tangent(&self, subtype: &RadioSubtype, moment: TangentMoment) -> Option<PathBuf> {
        sound_folder(subtype).map(|folder| self.in_folder(folder, moment))
    }

    /// Every remote push-to-talk sound, which the playback mixer decodes once at init.
    pub fn remote_sounds(&self) -> Vec<PathBuf> {
        SOUND_FOLDERS
            .iter()
            .flat_map(|folder| {
                [TangentMoment::RemoteStart, TangentMoment::RemoteEnd]
                    .map(|moment| self.in_folder(folder, moment))
            })
            .collect()
    }

    fn in_folder(&self, folder: &str, moment: TangentMoment) -> PathBuf {
        self.root
            .join(folder)
            .join(format!("{}.wav", moment.file_stem()))
    }
}

/// The HRIR sphere's path under `plugin_folder`: `tfar_hrir/IRC_1002_C.bin`, the dist layout.
pub fn hrir_sphere_path(plugin_folder: &Path) -> PathBuf {
    plugin_folder.join(HRIR_FOLDER).join(HRIR_FILE_NAME)
}
