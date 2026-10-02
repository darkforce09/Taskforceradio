//! Unit tests for the probe's command line.

use super::*;

fn arguments(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}

#[test]
fn a_library_alone_makes_the_default_calls() {
    let parsed = parse_arguments(arguments(&["task_force_radio_pipe_x64.dll"])).unwrap();
    assert_eq!(
        parsed.library_path,
        PathBuf::from("task_force_radio_pipe_x64.dll")
    );
    assert_eq!(parsed.calls, vec!["TS_INFO\tPING", "TS_INFO\tVERSION"]);
}

#[test]
fn named_calls_replace_the_defaults_in_order_with_tabs_expanded() {
    let parsed = parse_arguments(arguments(&[
        "ext.so",
        "TS_INFO\\tPING",
        "DFRAME~",
        "a\\tb\\tc",
    ]))
    .unwrap();
    assert_eq!(parsed.calls, vec!["TS_INFO\tPING", "DFRAME~", "a\tb\tc"]);
}

#[test]
fn a_literal_tab_passes_unchanged() {
    let parsed = parse_arguments(arguments(&["ext.so", "TS_INFO\tPING"])).unwrap();
    assert_eq!(parsed.calls, vec!["TS_INFO\tPING"]);
}

#[test]
fn no_library_or_a_help_flag_is_a_usage_error() {
    assert_eq!(parse_arguments(arguments(&[])), Err(USAGE));
    assert_eq!(parse_arguments(arguments(&["-h"])), Err(USAGE));
    assert_eq!(parse_arguments(arguments(&["--help"])), Err(USAGE));
    assert_eq!(parse_arguments(arguments(&[""])), Err(USAGE));
}

#[test]
fn only_the_tab_escape_is_expanded() {
    assert_eq!(expand_tab_escapes("a\\nb\\\\c"), "a\\nb\\\\c");
    assert_eq!(expand_tab_escapes("POS\\tx\\ty"), "POS\tx\ty");
}
