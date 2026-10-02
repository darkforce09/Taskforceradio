//! Unit tests for the 1.0 custom-variable reader, case by case against the original parser.

use super::*;

fn variable(id: u32, name: &str, value: &str) -> CustomVariable {
    CustomVariable {
        id,
        name: name.to_owned(),
        value: value.to_owned(),
    }
}

#[test]
fn a_session_tracker_array_reads_every_variable() {
    let text =
        r#"[[1,"group",3],[5,"faction","BLU_F"],[6,"isServer",false],[19,"version","1.0.0.1"]]"#;
    assert_eq!(
        parse_custom_variables(text),
        vec![
            variable(1, "group", "3"),
            variable(5, "faction", "BLU_F"),
            variable(6, "isServer", "false"),
            variable(19, "version", "1.0.0.1"),
        ]
    );
}

#[test]
fn whitespace_is_skipped_everywhere_even_inside_quoted_text() {
    let text = " [ [ 1 , \"group\"   , 3 ] ,\t[8, \"missionName\", \"Op Test\"]\r\n]";
    assert_eq!(
        parse_custom_variables(text),
        vec![
            variable(1, "group", "3"),
            variable(8, "missionName", "OpTest")
        ]
    );
}

#[test]
fn a_value_runs_to_the_next_closing_bracket_commas_included() {
    let text = r#"[[2,"pair","a,b"]]"#;
    assert_eq!(
        parse_custom_variables(text),
        vec![variable(2, "pair", "a,b")]
    );
}

#[test]
fn an_id_reads_like_stoi_with_trailing_junk_ignored_and_negatives_wrapped() {
    let text = r#"[[7x,"a","b"],[-1,"c","d"],[+3,"e","f"]]"#;
    assert_eq!(
        parse_custom_variables(text),
        vec![
            variable(7, "a", "b"),
            variable(u32::MAX, "c", "d"),
            variable(3, "e", "f")
        ]
    );
}

#[test]
fn an_id_stoi_refuses_voids_the_whole_array() {
    assert_eq!(
        parse_custom_variables(r#"[[1,"a","b"],["x","c","d"]]"#),
        Vec::new()
    );
    assert_eq!(parse_custom_variables(r#"[[,"a","b"]]"#), Vec::new());
    assert_eq!(
        parse_custom_variables(r#"[[99999999999,"a","b"]]"#),
        Vec::new()
    );
}

#[test]
fn text_that_ends_inside_a_variable_voids_the_array() {
    assert_eq!(parse_custom_variables(r#"[[1,"a","b"],[2,"c""#), Vec::new());
    assert_eq!(parse_custom_variables("[["), Vec::new());
    assert_eq!(parse_custom_variables("[[1"), Vec::new());
}

#[test]
fn a_flat_or_empty_array_holds_no_variable() {
    assert_eq!(parse_custom_variables(r#"[1,"a","b"]"#), Vec::new());
    assert_eq!(parse_custom_variables("[]"), Vec::new());
    assert_eq!(parse_custom_variables(""), Vec::new());
    assert_eq!(parse_custom_variables("no array here"), Vec::new());
}

#[test]
fn text_before_the_first_bracket_is_skipped() {
    assert_eq!(
        parse_custom_variables(r#"junk[[4,"n","v"]]"#),
        vec![variable(4, "n", "v")]
    );
}

#[test]
fn only_spaces_and_double_quotes_are_trimmed_from_names_and_values() {
    assert_eq!(
        parse_custom_variables(r#"[[1,"'quoted'","""v"""]]"#),
        vec![variable(1, "'quoted'", "v")]
    );
}
