//! Unit tests for the 0.9 tracking URL.

use super::*;

#[test]
fn the_game_path_goes_to_the_piwik_host_over_plain_http() {
    // The 0.9 session tracker (`fn_sessionTracker.sqf`) formats the whole query itself; the
    // plugin prefixes `http://nkey.piwik.pro/` and the request line encodes `"` and spaces.
    let path = r#"piwik.php?idsite=2&rec=1&url="radio.task-force.ru&action_name=start&rand=512.3&uid=765&cvar={"1":["group","3"], "2":["playableUnits","10"]}"#;
    let url = build_track_request_0_9(path).expect("the URL builds");
    assert_eq!(
        url.as_str(),
        concat!(
            "http://nkey.piwik.pro/piwik.php?idsite=2&rec=1&url=%22radio.task-force.ru",
            "&action_name=start&rand=512.3&uid=765",
            "&cvar={%221%22:[%22group%22,%223%22],%20%222%22:[%22playableUnits%22,%2210%22]}",
        )
    );
}

#[test]
fn a_leading_slash_is_optional() {
    let with = build_track_request_0_9("/piwik.php?a=1").expect("the URL builds");
    let without = build_track_request_0_9("piwik.php?a=1").expect("the URL builds");
    assert_eq!(with, without);
    assert_eq!(with.as_str(), "http://nkey.piwik.pro/piwik.php?a=1");
}

#[test]
fn a_path_without_query_keeps_no_question_mark() {
    let url = build_track_request_0_9("piwik.php").expect("the URL builds");
    assert_eq!(url.as_str(), "http://nkey.piwik.pro/piwik.php");
}
