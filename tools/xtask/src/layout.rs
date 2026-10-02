//! The structural laws of the workspace as data: categories, the edge matrix, the firewalls, the
//! unsafe allow-list, the feature rules and the folder lists the gates walk.
//!
//! **Role:** the one place the gates read the laws from; it mirrors
//! `documentation/program/laws_and_gates.md`, and when the two disagree that file is the spec and
//! this module is fixed.
//!
//! **Position:** tier 0 of `tools`; read by every gate under `commands/verify/` and by
//! `new-crate`; it calls nothing.
//!
//! **Signals and state:** none; constants and pure functions.
//!
//! **Invariants:** every category named here is a parent path (`crates/<category>`, `apps`,
//! `tools`); a crate name appears in at most one allow-list row of each table.

#[cfg(test)]
#[path = "tests/layout.rs"]
mod tests;

/// The foundation category: depends only on itself.
pub(crate) const FOUNDATION: &str = "crates/foundation";
/// The version 0.9 category.
pub(crate) const VERSION_0_9: &str = "crates/version_0_9";
/// The version 1.0 category.
pub(crate) const VERSION_1_0: &str = "crates/version_1_0";
/// The game link category.
pub(crate) const GAME_LINK: &str = "crates/game_link";
/// The deliverables category.
pub(crate) const APPS: &str = "apps";
/// The tooling category.
pub(crate) const TOOLS: &str = "tools";

/// Every category a member may declare, in the order of the laws.
pub(crate) const CATEGORIES: &[&str] = &[
    FOUNDATION,
    "crates/audio",
    "crates/teamspeak",
    GAME_LINK,
    "crates/services",
    "crates/interface",
    VERSION_0_9,
    VERSION_1_0,
    APPS,
    TOOLS,
];

/// The folders whose manifests must all be workspace members.
pub(crate) const MEMBER_ROOTS: &[&str] = &["apps", "crates", "tools"];

/// The folders whose every sub-folder carries a README.md.
pub(crate) const README_SPAN_ROOTS: &[&str] =
    &["apps", "crates", "tools", "documentation", "assets"];

/// Folder names exempt from the README span, with everything below them.
pub(crate) const README_EXEMPT_FOLDERS: &[&str] = &["tests", "target"];

/// The only `game_link` crate a version crate may use.
pub(crate) const GAME_LINK_PROTOCOL: &str = "tfar_game_link_protocol";

/// The two TeamSpeak plugin apps.
pub(crate) const PLUGIN_APPS: &[&str] = &["tfar_plugin_0_9", "tfar_plugin_1_0"];

/// The categories a crate in `from` may depend on, by the category edge matrix. Apps have
/// per-crate rules in [`app_allows`]; this returns the rule for an app not named there.
pub(crate) fn category_allows(from: &str, to: &str) -> bool {
    match from {
        FOUNDATION => to == FOUNDATION,
        "crates/audio" | "crates/teamspeak" | GAME_LINK => to == FOUNDATION || to == from,
        "crates/services" | "crates/interface" => to == FOUNDATION,
        VERSION_0_9 | VERSION_1_0 => {
            to == from
                || matches!(
                    to,
                    FOUNDATION
                        | "crates/audio"
                        | "crates/teamspeak"
                        | GAME_LINK
                        | "crates/services"
                        | "crates/interface"
                )
        }
        APPS | TOOLS => to.starts_with("crates/"),
        _ => false,
    }
}

/// Whether the app `app_name` may depend on a crate of category `to`.
pub(crate) fn app_allows(app_name: &str, to: &str) -> bool {
    match app_name {
        "tfar_plugin_0_9" => to.starts_with("crates/") && to != VERSION_1_0,
        "tfar_plugin_1_0" => to.starts_with("crates/") && to != VERSION_0_9,
        "tfar_extension_0_9" | "tfar_extension_1_0" => to == FOUNDATION || to == GAME_LINK,
        _ => category_allows(APPS, to),
    }
}

/// The version category a plugin app belongs to, so the version ban covers it.
pub(crate) fn version_category_of_app(app_name: &str) -> Option<&'static str> {
    match app_name {
        "tfar_plugin_0_9" => Some(VERSION_0_9),
        "tfar_plugin_1_0" => Some(VERSION_1_0),
        _ => None,
    }
}

/// The other version category of a version category.
pub(crate) fn other_version(category: &str) -> Option<&'static str> {
    match category {
        VERSION_0_9 => Some(VERSION_1_0),
        VERSION_1_0 => Some(VERSION_0_9),
        _ => None,
    }
}

/// The kind of a workspace dependency edge, as far as the edge laws care.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EdgeTable {
    /// `[dependencies]` or `[build-dependencies]`: bound by the matrix and the tier order.
    NormalOrBuild,
    /// `[dev-dependencies]`: exempt from the matrix and the tiers, never on apps.
    Development,
}

/// What is wrong with a workspace dependency edge, or `None` when the laws allow it.
pub(crate) fn edge_violation(
    from_name: &str,
    from_category: &str,
    to_name: &str,
    to_category: &str,
    table: EdgeTable,
) -> Option<String> {
    let own_version = if from_category == APPS {
        version_category_of_app(from_name)
    } else {
        Some(from_category)
    };
    if own_version.and_then(other_version) == Some(to_category) {
        return Some(format!(
            "`{to_name}` is in {to_category}: version_0_9 and version_1_0 never depend on each other"
        ));
    }
    if to_category == APPS {
        return Some(match (from_category, table) {
            (TOOLS, _) => format!("`{to_name}` is an app: tools never depend on apps"),
            (_, EdgeTable::Development) => {
                format!("`{to_name}` is an app: dev-dependencies never point at apps")
            }
            _ => format!("`{to_name}` is an app: nothing depends on an app"),
        });
    }
    if table == EdgeTable::Development {
        return None;
    }
    let allowed = if from_category == APPS {
        app_allows(from_name, to_category)
    } else {
        category_allows(from_category, to_category)
    };
    let is_version_crate = from_category == VERSION_0_9 || from_category == VERSION_1_0;
    if is_version_crate && to_category == GAME_LINK && to_name != GAME_LINK_PROTOCOL {
        return Some(format!(
            "`{to_name}` is in {GAME_LINK}: a version crate may use only {GAME_LINK_PROTOCOL} there"
        ));
    }
    (!allowed).then(|| {
        format!("the edge {from_category} -> {to_category} (`{to_name}`) is not in the category edge matrix")
    })
}

/// Who may use a firewalled external crate.
#[derive(Debug, Clone, Copy)]
pub(crate) enum FirewallScope {
    /// Only these packages.
    Crates(&'static [&'static str]),
    /// Every package of this category.
    Category(&'static str),
}

impl FirewallScope {
    /// Whether a package with this name and category is inside the scope.
    pub(crate) fn admits(self, package: &str, category: &str) -> bool {
        match self {
            Self::Crates(names) => names.contains(&package),
            Self::Category(allowed) => allowed == category,
        }
    }

    /// The scope in words, for a violation message.
    pub(crate) fn describe(self) -> String {
        match self {
            Self::Crates(names) => names.join(", "),
            Self::Category(category) => format!("the {category} crates"),
        }
    }
}

/// The external-crate firewalls: each crate may be a direct dependency only inside its scope.
pub(crate) const FIREWALLS: &[(&str, FirewallScope)] = &[
    (
        "windows-sys",
        FirewallScope::Crates(&["tfar_arma_extension_abi"]),
    ),
    ("rusqlite", FirewallScope::Crates(&["tfar_ts3_settings_db"])),
    ("ureq", FirewallScope::Crates(&["tfar_web_requests"])),
    ("eframe", FirewallScope::Crates(&["tfar_debug_window"])),
    ("egui", FirewallScope::Crates(&["tfar_debug_window"])),
    ("cc", FirewallScope::Crates(&["tfar_golden_vectors"])),
    ("sci-rs", FirewallScope::Crates(&["tfar_dsp_filters"])),
    ("biquad", FirewallScope::Crates(&["tfar_dsp_filters"])),
    ("hrtf", FirewallScope::Crates(&["tfar_hrtf"])),
    ("rustfft", FirewallScope::Crates(&["tfar_hrtf"])),
    ("realfft", FirewallScope::Crates(&["tfar_hrtf"])),
    ("hound", FirewallScope::Crates(&["tfar_playback_mixer"])),
    (
        "ts3plugin-sys",
        FirewallScope::Crates(&["tfar_ts3_host", "tfar_ts3_mock"]),
    ),
    ("postcard", FirewallScope::Category(GAME_LINK)),
    (
        "libloading",
        FirewallScope::Crates(&["tfar_ts3_mock", "tfar_extension_probe"]),
    ),
    ("tracy-client", FirewallScope::Crates(PLUGIN_APPS)),
    ("anyhow", FirewallScope::Category(TOOLS)),
    ("clap", FirewallScope::Crates(&["xtask"])),
    ("xshell", FirewallScope::Crates(&["xtask"])),
    ("cargo_metadata", FirewallScope::Crates(&["xtask"])),
    ("toml_edit", FirewallScope::Crates(&["xtask"])),
    ("sha2", FirewallScope::Crates(&["xtask"])),
    ("walkdir", FirewallScope::Crates(&["xtask"])),
    ("ignore", FirewallScope::Crates(&["xtask"])),
];

/// The external crate that must stay optional, behind the plugin apps' `tracy` feature.
pub(crate) const OPTIONAL_ONLY_EXTERNAL: &str = "tracy-client";

/// The crates that may lift `unsafe_code` at their crate root.
pub(crate) const UNSAFE_ALLOWED: &[&str] = &[
    "tfar_ts3_host",
    "tfar_arma_extension_abi",
    "tfar_plugin_0_9",
    "tfar_plugin_1_0",
    "tfar_extension_0_9",
    "tfar_extension_1_0",
    "tfar_ts3_mock",
];

/// The feature any crate may declare.
pub(crate) const FEATURE_ANYWHERE: &str = "test_fixtures";

/// The features only the plugin apps may declare.
pub(crate) const PLUGIN_APP_FEATURES: &[&str] = &["tracy", "api_profiler"];

/// The longest production `.rs` file, in lines.
pub(crate) const PRODUCTION_FILE_MAX_LINES: usize = 500;

/// The longest test `.rs` file (anything under a `tests` folder), in lines.
pub(crate) const TEST_FILE_MAX_LINES: usize = 1000;

/// The longest `lib.rs`, in lines.
pub(crate) const LIB_RS_MAX_LINES: usize = 80;

/// The line count from which a production file needs the four-point header.
pub(crate) const HEADER_REQUIRED_FROM_LINES: usize = 80;

/// The longest live document, in lines.
pub(crate) const LIVE_DOCUMENT_MAX_LINES: usize = 500;

/// The four labels of the module header, in order.
pub(crate) const MODULE_HEADER_LABELS: &[&str] = &[
    "**Role:**",
    "**Position:**",
    "**Signals and state:**",
    "**Invariants:**",
];

/// The only folders that may hold C or C++ sources.
pub(crate) const C_SOURCE_FOLDERS: &[&str] = &[
    "tools/tfar_golden_vectors/oracle/",
    "tools/tfar_golden_vectors/shim/",
];

/// File extensions of C and C++ sources and headers.
pub(crate) const C_SOURCE_EXTENSIONS: &[&str] = &["c", "h", "cc", "cpp", "cxx", "hh", "hpp", "hxx"];

/// File extensions banned outside `reference/`.
pub(crate) const BANNED_EXTENSIONS: &[&str] = &["py", "sh", "bash", "js", "ts"];

/// File names banned outside `reference/`.
pub(crate) const BANNED_FILE_NAMES: &[&str] = &["Makefile", "makefile", "GNUmakefile"];

/// The root Markdown files allowed outside `documentation/`.
pub(crate) const ROOT_MARKDOWN_FILES: &[&str] = &["CLAUDE.md", "AGENTS.md", "README.md"];

/// The Markdown file name allowed anywhere under `assets/`: third-party data attribution.
pub(crate) const ASSET_CREDITS_FILE: &str = "CREDITS.md";

/// The top-level folders a backticked path must start with for `link-check` to check it (a
/// root file name also qualifies).
pub(crate) const LINKABLE_TOP_FOLDERS: &[&str] = &[
    "apps",
    "crates",
    "tools",
    "documentation",
    "assets",
    "reference",
];

/// The read-only tree of the original mods.
pub(crate) const REFERENCE_FOLDER: &str = "reference";

/// The checksum list of the reference trees, relative to the repository root.
pub(crate) const REFERENCE_MANIFEST: &str = "reference/MANIFEST.sha256";

/// Paths under `reference/` the manifest leaves out, relative to `reference/`; a trailing `/`
/// marks a folder.
pub(crate) const REFERENCE_EXCLUDED: &[&str] = &["README.md", "MANIFEST.sha256", "tfar_1_0/.git/"];

/// The README standard whose kind table `readme-sections` reads.
pub(crate) const README_STANDARD: &str = "documentation/standards/readme_standard.md";

/// The three status lines a document may open with; the frozen one carries a date.
pub(crate) const STATUS_LIVE: &str = "**Status:** live";
/// The archived status line.
pub(crate) const STATUS_ARCHIVED: &str = "**Status:** archived";
/// The opening of a frozen record's status line, followed by `(YYYY-MM-DD)`.
pub(crate) const STATUS_FROZEN_PREFIX: &str = "**Status:** frozen record (";
