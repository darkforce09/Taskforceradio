//! What the debug window shows: titled sections, each a list of key/value rows or a table.
//!
//! **Role:** the immutable picture the plugin fills from its state (the client list with positions,
//! frequencies and radios, the game link, the settings) and publishes; the window draws the latest
//! one and never sees plugin state directly.
//!
//! **Position:** inside `tfar_debug_window`; built by `tfar_plugin_1_0`, stored by the handle,
//! read by `egui_window`.
//!
//! **Signals and state:** plain data; once published, a snapshot is shared behind an `Arc` and
//! never changes.
//!
//! **Invariants:** a table is as wide as its widest row or its column list, whichever is wider;
//! a missing cell reads as empty text, so a short row never hides data or indexes out of range.

#[cfg(test)]
#[path = "tests/debug_snapshot.rs"]
mod tests;

/// Everything the debug window shows at one moment, section by section, top to bottom.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DebugSnapshot {
    /// The sections in display order.
    pub sections: Vec<DebugSection>,
}

impl DebugSnapshot {
    /// An empty snapshot; the window then says there is nothing to show.
    pub fn new() -> Self {
        Self::default()
    }

    /// The snapshot with `section` appended.
    #[must_use]
    pub fn with_section(mut self, section: DebugSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Appends `section`.
    pub fn push_section(&mut self, section: DebugSection) {
        self.sections.push(section);
    }

    /// Whether the snapshot holds no section.
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }
}

/// One collapsible section of the window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugSection {
    /// The heading, unique within a snapshot so the window keeps each section's open state.
    pub title: String,
    /// The rows under the heading.
    pub content: SectionContent,
}

impl DebugSection {
    /// A section of key/value rows, such as the game link state or one client's details.
    pub fn key_values<K, V>(
        title: impl Into<String>,
        rows: impl IntoIterator<Item = (K, V)>,
    ) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            title: title.into(),
            content: SectionContent::KeyValues(
                rows.into_iter()
                    .map(|(key, value)| KeyValueRow {
                        key: key.into(),
                        value: value.into(),
                    })
                    .collect(),
            ),
        }
    }

    /// A section holding one table, such as the client list.
    pub fn table(title: impl Into<String>, table: DebugTable) -> Self {
        Self {
            title: title.into(),
            content: SectionContent::Table(table),
        }
    }
}

/// The rows of a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SectionContent {
    /// Two columns: a key and its value.
    KeyValues(Vec<KeyValueRow>),
    /// A table with a header row.
    Table(DebugTable),
}

/// One key/value row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyValueRow {
    /// What the value is.
    pub key: String,
    /// The value, already formatted.
    pub value: String,
}

/// A table: named columns and rows of formatted cells.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DebugTable {
    /// The column headings.
    pub columns: Vec<String>,
    /// The rows, top to bottom.
    pub rows: Vec<TableRow>,
}

impl DebugTable {
    /// An empty table with these column headings.
    pub fn new<C: Into<String>>(columns: impl IntoIterator<Item = C>) -> Self {
        Self {
            columns: columns.into_iter().map(Into::into).collect(),
            rows: Vec::new(),
        }
    }

    /// Appends a row in the normal colour.
    pub fn push_row<C: Into<String>>(&mut self, cells: impl IntoIterator<Item = C>) {
        self.rows.push(TableRow::new(cells, false));
    }

    /// Appends a highlighted row, as the original drew a talking client in red.
    pub fn push_highlighted_row<C: Into<String>>(&mut self, cells: impl IntoIterator<Item = C>) {
        self.rows.push(TableRow::new(cells, true));
    }

    /// The number of columns drawn: the headings or the widest row, whichever is more.
    pub fn width(&self) -> usize {
        self.rows
            .iter()
            .map(|row| row.cells.len())
            .fold(self.columns.len(), usize::max)
    }

    /// The heading of column `index`, empty past the named ones.
    pub fn column(&self, index: usize) -> &str {
        self.columns.get(index).map_or("", String::as_str)
    }
}

/// One table row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableRow {
    /// The cells, left to right.
    pub cells: Vec<String>,
    /// Whether the row stands out, for example a client talking now.
    pub is_highlighted: bool,
}

impl TableRow {
    /// A row of these cells.
    pub fn new<C: Into<String>>(cells: impl IntoIterator<Item = C>, is_highlighted: bool) -> Self {
        Self {
            cells: cells.into_iter().map(Into::into).collect(),
            is_highlighted,
        }
    }

    /// The text of cell `index`, empty past the row's end.
    pub fn cell(&self, index: usize) -> &str {
        self.cells.get(index).map_or("", String::as_str)
    }
}
