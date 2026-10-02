//! Builds the golden-vector oracle: the original C++ filters and compressor, FAudio's F3DAudio and
//! the `extern "C"` shim over them, as one static library the crate links.
//!
//! **Role:** compiles DSPFilters and simpleSource straight from `reference/tfar_1_0/ts/include/`,
//! the vendored `oracle/F3DAudio.c` and `shim/oracle_shim.cpp` with `cc`, so the tests can run the
//! originals live beside the Rust audio crates.
//!
//! **Position:** the build script of `tfar_golden_vectors`; nothing else builds C or C++ in the
//! workspace, and no deliverable links the library it makes.
//!
//! **Signals and state:** writes only into `OUT_DIR`: the alias headers and the objects. The
//! original sources include `"DspFilters/…"` while their folder is the lowercase `dspfilters`, so
//! `OUT_DIR/include_alias/DspFilters/<name>.h` holds one `#include` of each original header by its
//! absolute path; the originals are read in place and never copied or changed.
//!
//! **Invariants:** nothing under `reference/` is written; a missing `reference/` fails the build
//! with a message naming the folder and how to restore it.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// The DSPFilters translation units the TFAR designs need, in `dspfilters/source/`.
const DSPFILTERS_SOURCES: &[&str] = &[
    "Biquad.cpp",
    "Butterworth.cpp",
    "Cascade.cpp",
    "Design.cpp",
    "Filter.cpp",
    "Param.cpp",
    "PoleFilter.cpp",
    "RBJ.cpp",
    "RootFinder.cpp",
    "State.cpp",
];

/// The simpleSource translation units of the compressor, in `simpleSource/`.
const SIMPLE_SOURCE_SOURCES: &[&str] = &["SimpleComp.cpp", "SimpleEnvelope.cpp"];

type BuildResult<T> = Result<T, Box<dyn Error>>;

fn main() -> BuildResult<()> {
    let crate_folder = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let out_folder = PathBuf::from(std::env::var("OUT_DIR")?);
    let include_folder = crate_folder.join("../../reference/tfar_1_0/ts/include");
    let include_folder = include_folder.canonicalize().map_err(|error| {
        format!(
            "tfar_golden_vectors needs the original sources in reference/tfar_1_0/ts/include \
             ({}: {error}); restore reference/ as reference/README.md describes",
            include_folder.display()
        )
    })?;
    let dspfilters_folder = include_folder.join("dspfilters");
    let simple_source_folder = include_folder.join("simpleSource");
    let oracle_folder = crate_folder.join("oracle");
    let shim_folder = crate_folder.join("shim");

    let alias_folder = out_folder.join("include_alias");
    write_dspfilters_aliases(&dspfilters_folder, &alias_folder.join("DspFilters"))?;

    for watched in [
        &dspfilters_folder,
        &simple_source_folder,
        &oracle_folder,
        &shim_folder,
    ] {
        println!("cargo:rerun-if-changed={}", watched.display());
    }
    println!("cargo:rerun-if-changed=build.rs");

    let mut cpp = cc::Build::new();
    cpp.cpp(true)
        .std("c++17")
        .warnings(false)
        .include(&alias_folder)
        .include(&simple_source_folder)
        .include(&oracle_folder)
        .include(&shim_folder)
        // `State.h` and `Filter.h` use `std::array` without including `<array>`; the originals
        // built with MSVC, whose headers pull it in, so every unit gets it forced in.
        .flag("-include")
        .flag("array");
    for source in DSPFILTERS_SOURCES {
        cpp.file(dspfilters_folder.join("source").join(source));
    }
    for source in SIMPLE_SOURCE_SOURCES {
        cpp.file(simple_source_folder.join(source));
    }
    cpp.file(shim_folder.join("oracle_shim.cpp"));
    cpp.try_compile("tfar_golden_vectors_oracle_cpp")?;

    cc::Build::new()
        .std("c11")
        .warnings(false)
        .include(&oracle_folder)
        .include(&shim_folder)
        .file(oracle_folder.join("F3DAudio.c"))
        .try_compile("tfar_golden_vectors_oracle_c")?;
    Ok(())
}

/// Writes one alias header per original DSPFilters header into `alias_folder`, each including the
/// original by its absolute path, so `#include "DspFilters/Biquad.h"` finds
/// `dspfilters/Biquad.h`.
fn write_dspfilters_aliases(dspfilters_folder: &Path, alias_folder: &Path) -> BuildResult<()> {
    fs::create_dir_all(alias_folder)?;
    for entry in fs::read_dir(dspfilters_folder)? {
        let path = entry?.path();
        if path.extension().is_none_or(|extension| extension != "h") {
            continue;
        }
        let Some(name) = path.file_name() else {
            continue;
        };
        // An #include path takes no escapes, so a path holding a quote cannot be written.
        let original = path
            .to_str()
            .filter(|original| !original.contains('"'))
            .ok_or("a DSPFilters header path is not UTF-8 or holds a quote")?;
        fs::write(
            alias_folder.join(name),
            format!("#include \"{original}\"\n"),
        )?;
    }
    Ok(())
}
