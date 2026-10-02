//! Unit tests for the snapshot view, drawn headless through an egui context with no window.

use super::*;
use crate::debug_snapshot::DebugTable;

/// Runs one headless egui pass that draws `snapshot` and returns the text of every label drawn.
fn drawn_texts(snapshot: &DebugSnapshot) -> Vec<String> {
    let context = egui::Context::default();
    let mut texts = Vec::new();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1200.0, 2000.0),
        )),
        ..egui::RawInput::default()
    };
    let mut output = context.run_ui(input, |ui| show_snapshot(ui, snapshot));
    // No painter uploads the font atlas here, so the texture changes are dropped on purpose.
    output.textures_delta.clear();
    for clipped in output.shapes {
        collect_texts(&clipped.shape, &mut texts);
    }
    texts
}

fn collect_texts(shape: &egui::epaint::Shape, texts: &mut Vec<String>) {
    match shape {
        egui::epaint::Shape::Text(text) => texts.push(text.galley.text().to_owned()),
        egui::epaint::Shape::Vec(shapes) => {
            for inner in shapes {
                collect_texts(inner, texts);
            }
        }
        _ => {}
    }
}

fn sample_snapshot() -> DebugSnapshot {
    let mut clients = DebugTable::new(["nickname", "distance", "frequency"]);
    clients.push_row(["Alpha", "12.5 m", "45.1"]);
    clients.push_highlighted_row(["Bravo", "40.0 m", "52.3", "talking"]);
    DebugSnapshot::new()
        .with_section(DebugSection::key_values(
            "Game link",
            [("session", "up"), ("data frame", "1234")],
        ))
        .with_section(DebugSection::table("Clients", clients))
}

#[test]
fn every_section_title_key_value_and_cell_is_drawn() {
    let texts = drawn_texts(&sample_snapshot());
    for expected in [
        "Game link",
        "session",
        "up",
        "data frame",
        "1234",
        "Clients",
        "nickname",
        "distance",
        "frequency",
        "Alpha",
        "12.5 m",
        "Bravo",
        "52.3",
        "talking",
    ] {
        assert!(
            texts.iter().any(|text| text == expected),
            "{expected:?} not drawn in {texts:?}"
        );
    }
}

#[test]
fn an_empty_snapshot_says_there_is_nothing_to_show() {
    let texts = drawn_texts(&DebugSnapshot::new());
    assert_eq!(texts, [NOTHING_TO_SHOW]);
}

#[test]
fn the_highlight_is_the_originals_talking_red() {
    assert_eq!(HIGHLIGHT_COLOUR, egui::Color32::from_rgb(255, 0, 51));
}
