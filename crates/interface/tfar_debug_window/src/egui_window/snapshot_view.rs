//! Draws a [`DebugSnapshot`] with egui.
//!
//! **Role:** lays a snapshot out as one `CollapsingHeader` per section, its rows in a striped
//! `Grid`, the whole page in a vertical scroll area.
//!
//! **Position:** inside `egui_window`; called by `eframe_backend`'s app on every frame, and by the
//! tests on a headless egui context.
//!
//! **Signals and state:** runs on the UI thread only; egui keeps each section's open state under
//! an id salted with its title.
//!
//! **Invariants:** every section, key, value, heading and cell is drawn; a table draws as many
//! columns as its widest row or its headings, missing cells as empty text.

#[cfg(test)]
#[path = "tests/snapshot_view.rs"]
mod tests;

use eframe::egui;

use crate::debug_snapshot::{DebugSection, DebugSnapshot, DebugTable, KeyValueRow, SectionContent};

/// The colour of a highlighted row: the original's talking-client red, `(1.0, 0.0, 0.2)`.
const HIGHLIGHT_COLOUR: egui::Color32 = egui::Color32::from_rgb(255, 0, 51);

/// What an empty snapshot shows.
const NOTHING_TO_SHOW: &str = "Nothing to show yet: the plugin has published no state.";

/// Draws every section of `snapshot` into `ui`.
pub(crate) fn show_snapshot(ui: &mut egui::Ui, snapshot: &DebugSnapshot) {
    if snapshot.is_empty() {
        ui.weak(NOTHING_TO_SHOW);
        return;
    }
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for section in &snapshot.sections {
                show_section(ui, section);
            }
        });
}

/// One collapsible section, open by default, keyed by its title.
fn show_section(ui: &mut egui::Ui, section: &DebugSection) {
    egui::CollapsingHeader::new(section.title.as_str())
        .id_salt(section.title.as_str())
        .default_open(true)
        .show(ui, |ui| match &section.content {
            SectionContent::KeyValues(rows) => show_key_values(ui, &section.title, rows),
            SectionContent::Table(table) => show_table(ui, &section.title, table),
        });
}

/// Two columns, the key in strong text.
fn show_key_values(ui: &mut egui::Ui, title: &str, rows: &[KeyValueRow]) {
    egui::Grid::new(("tfar_debug_key_values", title))
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            for row in rows {
                ui.strong(row.key.as_str());
                ui.label(row.value.as_str());
                ui.end_row();
            }
        });
}

/// A header row of column names, then each row's cells, highlighted rows in red.
fn show_table(ui: &mut egui::Ui, title: &str, table: &DebugTable) {
    let width = table.width();
    egui::Grid::new(("tfar_debug_table", title))
        .num_columns(width)
        .striped(true)
        .show(ui, |ui| {
            for index in 0..width {
                ui.strong(table.column(index));
            }
            ui.end_row();
            for row in &table.rows {
                for index in 0..width {
                    let cell = row.cell(index);
                    if row.is_highlighted {
                        ui.colored_label(HIGHLIGHT_COLOUR, cell);
                    } else {
                        ui.label(cell);
                    }
                }
                ui.end_row();
            }
        });
}
