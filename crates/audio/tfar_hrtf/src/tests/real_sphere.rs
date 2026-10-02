//! The real IRCAM sphere from `assets/hrir/`, loaded once for every test in the crate.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use crate::hrir_library::HrirLibrary;

/// The sphere file the plugins ship, found from this crate's manifest folder.
pub(crate) fn real_sphere_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../assets/hrir/IRC_1002_C.bin")
}

/// The real sphere, loaded on first use and shared by every test thread.
pub(crate) fn real_library() -> Arc<HrirLibrary> {
    static LIBRARY: OnceLock<Arc<HrirLibrary>> = OnceLock::new();
    LIBRARY
        .get_or_init(|| HrirLibrary::load(real_sphere_path()).expect("the shipped sphere loads"))
        .clone()
}
