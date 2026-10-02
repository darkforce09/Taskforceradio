//! The parent README edit of `new-crate`: one Contents line for the new crate folder.
//!
//! **Role:** adds `<name>/  <role>` to the Contents block of the category's README and redraws the
//! block's entries in case-insensitive name order with aligned roles.
//!
//! **Position:** called by `scaffold`; finds the block with `readme_span`.
//!
//! **Signals and state:** none; text in, text out.
//!
//! **Invariants:** only the entry lines of the Contents block change; the root line and every
//! other line of the README stay byte for byte.

use anyhow::{Context, Result, bail};

use crate::support::readme_span;

/// The README text with an entry for the folder `name/` added to its Contents block.
pub(crate) fn with_contents_entry(readme_text: &str, name: &str, role: &str) -> Result<String> {
    let block = readme_span::contents_block(readme_text)
        .context("the parent README has no Contents block")?;
    let (root_line, _) = block
        .root_line
        .context("the parent README's Contents block is empty")?;
    let lines: Vec<&str> = readme_text.lines().collect();
    let first_entry = root_line;
    let end = lines
        .iter()
        .enumerate()
        .skip(first_entry)
        .find(|(_, line)| line.trim_start().starts_with("```"))
        .map(|(index, _)| index)
        .context("the Contents block never closes")?;
    let mut entries: Vec<(String, String)> = Vec::new();
    for line in lines.get(first_entry..end).unwrap_or(&[]) {
        let body = line
            .strip_prefix("├── ")
            .or_else(|| line.strip_prefix("└── "))
            .unwrap_or(line);
        let Some((entry_name, entry_role)) = body.split_once("  ") else {
            continue;
        };
        entries.push((entry_name.trim().to_owned(), entry_role.trim().to_owned()));
    }
    let new_name = format!("{name}/");
    if entries
        .iter()
        .any(|(entry_name, _)| *entry_name == new_name)
    {
        bail!("the parent README already lists `{new_name}`");
    }
    entries.push((new_name, role.to_owned()));
    entries.sort_by_key(|(entry_name, _)| entry_name.trim_end_matches('/').to_lowercase());
    let width = entries
        .iter()
        .map(|(entry_name, _)| entry_name.chars().count())
        .max()
        .unwrap_or(0)
        + 2;
    let count = entries.len();
    let rendered: Vec<String> = entries
        .iter()
        .enumerate()
        .map(|(index, (entry_name, entry_role))| {
            let branch = if index + 1 == count {
                "└── "
            } else {
                "├── "
            };
            let padding = " ".repeat(width.saturating_sub(entry_name.chars().count()));
            format!("{branch}{entry_name}{padding}{entry_role}")
        })
        .collect();
    let mut output: Vec<String> = lines
        .get(..first_entry)
        .unwrap_or(&[])
        .iter()
        .map(|line| (*line).to_owned())
        .collect();
    output.extend(rendered);
    output.extend(
        lines
            .get(end..)
            .unwrap_or(&[])
            .iter()
            .map(|line| (*line).to_owned()),
    );
    let mut text = output.join("\n");
    if readme_text.ends_with('\n') {
        text.push('\n');
    }
    Ok(text)
}
