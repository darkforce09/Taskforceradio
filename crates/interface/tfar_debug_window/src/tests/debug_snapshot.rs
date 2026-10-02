//! Unit tests for the snapshot model.

use super::*;

fn client_table() -> DebugTable {
    let mut table = DebugTable::new(["nickname", "position", "frequency"]);
    table.push_row(["Alpha", "[10, 20, 0]", "45.1"]);
    table.push_highlighted_row(["Bravo", "[12, 22, 0]", "45.1"]);
    table
}

#[test]
fn a_new_snapshot_is_empty() {
    assert!(DebugSnapshot::new().is_empty());
    assert_eq!(DebugSnapshot::new(), DebugSnapshot::default());
}

#[test]
fn sections_keep_the_order_they_were_added_in() {
    let mut snapshot = DebugSnapshot::new()
        .with_section(DebugSection::key_values("Game link", [("session", "up")]))
        .with_section(DebugSection::table("Clients", client_table()));
    snapshot.push_section(DebugSection::key_values(
        "Settings",
        [("full_duplex", "true")],
    ));
    let titles: Vec<&str> = snapshot
        .sections
        .iter()
        .map(|section| section.title.as_str())
        .collect();
    assert_eq!(titles, ["Game link", "Clients", "Settings"]);
    assert!(!snapshot.is_empty());
}

#[test]
fn a_key_value_section_holds_its_rows_in_order() {
    let section = DebugSection::key_values(
        "Game link",
        [
            ("session", "up".to_owned()),
            ("last message", "0.2 s ago".to_owned()),
        ],
    );
    assert_eq!(
        section.content,
        SectionContent::KeyValues(vec![
            KeyValueRow {
                key: "session".to_owned(),
                value: "up".to_owned()
            },
            KeyValueRow {
                key: "last message".to_owned(),
                value: "0.2 s ago".to_owned()
            },
        ])
    );
}

#[test]
fn a_table_row_records_whether_it_is_highlighted() {
    let table = client_table();
    assert_eq!(table.rows.len(), 2);
    assert!(!table.rows[0].is_highlighted);
    assert!(table.rows[1].is_highlighted);
    assert_eq!(table.rows[1].cell(0), "Bravo");
}

#[test]
fn a_table_is_as_wide_as_its_widest_row_or_its_headings() {
    let mut table = DebugTable::new(["a", "b"]);
    assert_eq!(table.width(), 2);
    table.push_row(["1"]);
    assert_eq!(table.width(), 2);
    table.push_row(["1", "2", "3"]);
    assert_eq!(table.width(), 3);
    assert_eq!(DebugTable::default().width(), 0);
}

#[test]
fn a_missing_cell_or_heading_reads_as_empty_text() {
    let mut table = DebugTable::new(["only"]);
    table.push_row(["x", "y"]);
    assert_eq!(table.column(0), "only");
    assert_eq!(table.column(1), "");
    assert_eq!(table.rows[0].cell(1), "y");
    assert_eq!(table.rows[0].cell(5), "");
}
