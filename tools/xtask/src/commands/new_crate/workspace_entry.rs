//! The root manifest edit of `new-crate`: the new crate's `[workspace.dependencies]` path entry,
//! placed in its category's group.
//!
//! **Role:** inserts `name = { path = "<category>/<name>" }` after the last entry whose path lies
//! in the same category, keeping every other key, comment and banner as it was.
//!
//! **Position:** called by `scaffold`; edits the text of the root `Cargo.toml` with `toml_edit`.
//!
//! **Signals and state:** none; text in, text out.
//!
//! **Invariants:** an existing entry of the same name is an error, never overwritten; a category
//! with no entry yet gets a `# <category>:` banner after the last path entry.

use anyhow::{Context, Result, bail};
use toml_edit::{DocumentMut, InlineTable, Item, Key, Table, Value};

/// The root manifest text with the new path dependency inserted.
pub(crate) fn with_workspace_entry(
    manifest_text: &str,
    category: &str,
    name: &str,
) -> Result<String> {
    let mut document: DocumentMut = manifest_text
        .parse()
        .context("the root Cargo.toml does not parse")?;
    let dependencies = document
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(Item::as_table)
        .context("the root Cargo.toml has no [workspace.dependencies] table")?;
    if dependencies.contains_key(name) {
        bail!("[workspace.dependencies] already lists `{name}`");
    }
    let path_of = |item: &Item| {
        item.as_table_like()
            .and_then(|table| table.get("path"))
            .and_then(Item::as_str)
            .map(ToOwned::to_owned)
    };
    let keys: Vec<String> = dependencies.iter().map(|(key, _)| key.to_owned()).collect();
    let category_prefix = format!("{category}/");
    let last_in_category = keys.iter().rposition(|key| {
        dependencies
            .get(key)
            .and_then(path_of)
            .is_some_and(|path| path.starts_with(&category_prefix))
    });
    let last_path_entry = keys
        .iter()
        .rposition(|key| dependencies.get(key).and_then(path_of).is_some());
    let insert_after = last_in_category.or(last_path_entry);

    let mut entry = InlineTable::new();
    entry.insert("path", Value::from(format!("{category}/{name}")));
    entry.fmt();
    let mut new_key = Key::new(name);
    if last_in_category.is_none() {
        new_key
            .leaf_decor_mut()
            .set_prefix(format!("\n# {category}:\n"));
    }

    let mut rebuilt = Table::new();
    rebuilt.set_implicit(dependencies.is_implicit());
    *rebuilt.decor_mut() = dependencies.decor().clone();
    if let Some(position) = dependencies.position() {
        rebuilt.set_position(Some(position));
    }
    if insert_after.is_none() {
        rebuilt.insert_formatted(&new_key, Item::Value(Value::InlineTable(entry.clone())));
    }
    for (index, key) in keys.iter().enumerate() {
        let Some((formatted_key, item)) = dependencies.get_key_value(key) else {
            continue;
        };
        rebuilt.insert_formatted(formatted_key, item.clone());
        if insert_after == Some(index) {
            rebuilt.insert_formatted(&new_key, Item::Value(Value::InlineTable(entry.clone())));
        }
    }
    let workspace = document
        .get_mut("workspace")
        .and_then(Item::as_table_mut)
        .context("the root Cargo.toml has no [workspace] table")?;
    workspace.insert("dependencies", Item::Table(rebuilt));
    Ok(document.to_string())
}
