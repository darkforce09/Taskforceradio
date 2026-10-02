//! The files of a new crate skeleton: manifest, READMEs, crate root, prelude, error type, the
//! role module and its sibling test.
//!
//! **Role:** renders every file of a skeleton crate from a [`CrateSpec`], in the shape the
//! workspace's existing skeletons have, so the new crate passes every gate as written.
//!
//! **Position:** used by `scaffold` (and by the test fixtures, which build their crates with it).
//!
//! **Signals and state:** none; pure functions returning text.
//!
//! **Invariants:** every rendered line is within 100 characters where the text allows; an app is
//! a `cdylib` + `rlib` with `#![allow(unsafe_code)]` under a reason comment; a library or tool is
//! a plain library; dependencies are `workspace = true` with a why comment.

#[cfg(test)]
#[path = "tests/crate_templates.rs"]
mod tests;

use std::fmt::Write as _;

use crate::layout::APPS;

/// What a new crate is.
#[derive(Debug, Clone)]
pub(crate) struct CrateSpec {
    /// The category: `crates/<category>`, `apps` or `tools`.
    pub(crate) category: String,
    /// The package and folder name.
    pub(crate) name: String,
    /// The computed tier.
    pub(crate) tier: u64,
    /// One sentence on what the crate is for, ending with a period.
    pub(crate) summary: String,
    /// The workspace crates it depends on.
    pub(crate) dependencies: Vec<String>,
}

impl CrateSpec {
    /// The crate folder, repository-relative.
    pub(crate) fn folder(&self) -> String {
        format!("{}/{}", self.category, self.name)
    }

    fn is_app(&self) -> bool {
        self.category == APPS
    }

    fn depends_on_text(&self) -> String {
        if self.dependencies.is_empty() {
            "no workspace crate".to_owned()
        } else {
            self.dependencies
                .iter()
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}

/// The human title of a crate name: `tfar_sample_buffer` becomes `Sample buffer`.
pub(crate) fn title_of(name: &str) -> String {
    let words = name.strip_prefix("tfar_").unwrap_or(name).replace('_', " ");
    let mut characters = words.chars();
    characters
        .next()
        .map(|first| first.to_uppercase().chain(characters).collect())
        .unwrap_or_default()
}

/// Greedy word wrap: the words of `text` in lines of at most `width` characters.
pub(crate) fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if !current.is_empty() && current.chars().count() + 1 + word.chars().count() > width {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn prefixed(prefix: &str, text: &str, width: usize) -> String {
    let mut out = String::new();
    for line in wrap(text, width.saturating_sub(prefix.len())) {
        let _ = writeln!(out, "{prefix}{line}");
    }
    out
}

/// Every file of the skeleton: (path relative to the crate folder, contents).
pub(crate) fn crate_files(spec: &CrateSpec) -> Vec<(String, String)> {
    vec![
        ("Cargo.toml".to_owned(), manifest(spec)),
        ("README.md".to_owned(), crate_readme(spec)),
        ("src/README.md".to_owned(), source_readme(spec)),
        ("src/lib.rs".to_owned(), lib_rs(spec)),
        ("src/prelude.rs".to_owned(), prelude_rs(spec)),
        ("src/error.rs".to_owned(), error_rs(spec)),
        ("src/crate_role.rs".to_owned(), crate_role_rs(spec)),
        (
            "src/tests/crate_role.rs".to_owned(),
            crate_role_test_rs(spec),
        ),
    ]
}

fn manifest(spec: &CrateSpec) -> String {
    let mut out = prefixed(
        "# ",
        &format!("{} No crate depends on it yet.", spec.summary),
        100,
    );
    let description = spec.summary.replace('\\', "\\\\").replace('"', "\\\"");
    let _ = write!(
        out,
        "[package]\nname = \"{name}\"\ndescription = \"{description}\"\nversion.workspace = true\n\
         edition.workspace = true\nrust-version.workspace = true\nlicense.workspace = true\n\
         publish.workspace = true\n\n[package.metadata.layout]\ncategory = \"{category}\"\n\
         tier = {tier}\ntargets = \"any\"\n\n",
        name = spec.name,
        category = spec.category,
        tier = spec.tier,
    );
    if spec.is_app() {
        let _ = write!(
            out,
            "# The cdylib is the file the host loads; the rlib lets host tests link the same code.\n\
             [lib]\nname = \"{}\"\ncrate-type = [\"cdylib\", \"rlib\"]\n\n",
            spec.name
        );
    }
    out.push_str("[dependencies]\n# Derives the crate's `Error`.\nthiserror.workspace = true\n");
    for dependency in &spec.dependencies {
        let _ = write!(
            out,
            "# The `{dependency}` crate this one builds on.\n{dependency}.workspace = true\n"
        );
    }
    out.push_str("\n[lints]\nworkspace = true\n");
    out
}

fn crate_readme(spec: &CrateSpec) -> String {
    let folder = spec.folder();
    let kind_word = if spec.is_app() {
        "cdylib + rlib"
    } else {
        "lib"
    };
    let mut out = format!("# {}\n\n", title_of(&spec.name));
    out.push_str(&wrap(&format!("The `{}` crate: {}", spec.name, spec.summary), 100).join("\n"));
    let _ = write!(
        out,
        "\n\n## Contents\n\n```text\n{folder}/\n\
         ├── Cargo.toml  the `{name}` package ({kind_word}): tier {tier}, workspace lints and dependencies\n\
         └── src/        the crate root, the role constant, the error type and the prelude\n```\n\n\
         ## How it works\n\nThe code is not written yet: `src/lib.rs` carries the module header and re-exports,\n\
         `src/crate_role.rs` states the crate's role in one constant, and `src/error.rs` holds the\n\
         `thiserror` error type.\n\n## Getting started\n\nRun from the repository root:\n\n```bash\n\
         cargo test -p {name}  # the unit tests\n```\n\n## Configuration\n\nNone: the crate reads no setting.\n\n",
        name = spec.name,
        tier = spec.tier,
    );
    if spec.is_app() {
        out.push_str("## Exports\n\nNone yet: the crate exports no C symbol.\n\n");
    } else {
        out.push_str("## Public surface\n\n- `CRATE_ROLE`, `Error`, `Result` and `prelude`, re-exported at the crate root.\n\n");
    }
    let _ = write!(
        out,
        "## Boundaries\n\n- Depends on: {depends}; `thiserror` for the error type.\n- Used by: nothing.\n\
         - Rules: tier {tier} in `{category}`: every workspace dependency has a lower tier and follows\n  \
         the category edge matrix (`cargo xtask verify crate-tiers`).\n",
        depends = spec.depends_on_text(),
        tier = spec.tier,
        category = spec.category,
    );
    out
}

fn source_readme(spec: &CrateSpec) -> String {
    format!(
        "# {title} source\n\nThe source of `{name}`: the crate's role constant, its error type and its prelude. The\n\
         crate's own code is not written yet.\n\n## Contents\n\n```text\n{folder}/src/\n\
         ├── crate_role.rs  `CRATE_ROLE`: what the crate is for, in one sentence\n\
         ├── error.rs       `Error`, the `thiserror` enum, and `Result`\n\
         ├── lib.rs         the crate root: module header, `mod` lines and re-exports\n\
         ├── prelude.rs     `CRATE_ROLE` for glob import\n\
         └── tests/         unit tests of `CRATE_ROLE`\n```\n\n## How it works\n\n\
         `lib.rs` declares the private modules `crate_role` and `error` and re-exports `CRATE_ROLE`, `Error`\n\
         and `Result` at the crate root; `prelude.rs` re-exports `CRATE_ROLE` only, so a glob import never\n\
         shadows `std`'s `Result`. `crate_role.rs` attaches its sibling test file `tests/crate_role.rs` with\n\
         `#[path]`.\n\n## Public surface\n\n- `CRATE_ROLE`, `Error`, `Result` and `prelude`, re-exported at the crate root.\n\n\
         ## Boundaries\n\n- Depends on: `thiserror`.\n- Used by: nothing.\n\
         - Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests\n  \
         live in `tests/` (`cargo xtask verify test-placement`).\n",
        title = title_of(&spec.name),
        name = spec.name,
        folder = spec.folder(),
    )
}

fn lib_rs(spec: &CrateSpec) -> String {
    let mut out = prefixed("//! ", &spec.summary, 100);
    out.push_str("//!\n");
    out.push_str(&prefixed(
        "//! ",
        "**Role:** the code is not written yet; `CRATE_ROLE` states what the crate is for.",
        100,
    ));
    out.push_str("//!\n");
    out.push_str(&prefixed(
        "//! ",
        &format!(
            "**Position:** `{}`, tier {}, depending on {}; no crate depends on it yet.",
            spec.category,
            spec.tier,
            spec.depends_on_text()
        ),
        100,
    ));
    out.push_str("//!\n//! **Signals and state:** none; the crate holds no state yet.\n//!\n");
    out.push_str(
        "//! **Invariants:** the crate exports only its role, its error type and its prelude.\n\n",
    );
    if spec.is_app() {
        out.push_str(
            "// The C exports the host loads live in this crate; each `unsafe` block carries a `// SAFETY:`\n\
             // comment.\n#![allow(unsafe_code)]\n\n",
        );
    }
    out.push_str("mod crate_role;\nmod error;\npub mod prelude;\n\npub use crate_role::CRATE_ROLE;\npub use error::{Error, Result};\n");
    out
}

fn prelude_rs(spec: &CrateSpec) -> String {
    format!(
        "//! The names a caller of `{name}` imports with `use {name}::prelude::*;`.\n//!\n\
         //! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.\n\n\
         pub use crate::crate_role::CRATE_ROLE;\n",
        name = spec.name
    )
}

fn error_rs(spec: &CrateSpec) -> String {
    format!(
        "//! The error type of `{name}`.\n\n/// Every failure `{name}` reports.\n\
         #[derive(Debug, thiserror::Error)]\npub enum Error {{\n    \
         /// The operation asked for is not offered by this crate.\n    \
         #[error(\"{name} does not offer the operation `{{operation}}`\")]\n    \
         UnsupportedOperation {{\n        /// The name of the operation that was asked for.\n        \
         operation: &'static str,\n    }},\n}}\n\n\
         /// The result of a fallible `{name}` operation.\npub type Result<T> = core::result::Result<T, Error>;\n",
        name = spec.name
    )
}

fn crate_role_rs(spec: &CrateSpec) -> String {
    let role = format!("{}: {}", spec.name, spec.summary)
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let chunks = wrap(&role, 64);
    let mut constant = String::from("pub const CRATE_ROLE: &str = \"");
    for (index, chunk) in chunks.iter().enumerate() {
        if index > 0 {
            constant.push_str(" \\\n     ");
        }
        constant.push_str(chunk);
    }
    constant.push_str("\";\n");
    format!(
        "//! The one-line statement of what `{name}` is for, kept beside the code it describes.\n\n\
         #[cfg(test)]\n#[path = \"tests/crate_role.rs\"]\nmod tests;\n\n\
         /// What `{name}` is for, in one sentence.\n{constant}",
        name = spec.name
    )
}

fn crate_role_test_rs(spec: &CrateSpec) -> String {
    format!(
        "//! Unit tests for `CRATE_ROLE` in `{name}`.\n\nuse super::*;\n\n#[test]\n\
         fn crate_role_names_the_crate_it_describes() {{\n    \
         assert!(CRATE_ROLE.starts_with(\"{name}: \"));\n}}\n",
        name = spec.name
    )
}
