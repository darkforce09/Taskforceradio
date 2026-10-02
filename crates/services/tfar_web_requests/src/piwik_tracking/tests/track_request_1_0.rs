//! Unit tests for the 1.0 tracking URL, against strings derived by hand from
//! `TFAR::trackPiwik` in `reference/tfar_1_0/ts/src/task_force_radio.cpp`.

use super::*;

const SAMPLE_UID: &str = "76561198012345678";
const SAMPLE_CUSTOM_VARS: &str =
    r#"[[1,"group",3],[5, "faction", "BLU_F"],[8,"missionName","Op Test"]]"#;

#[test]
fn a_session_tracker_message_builds_the_originals_query() {
    // The derivation, step by step through the C++:
    //
    // 1. `parseCustomVariables` skips all whitespace (`istream >> char`), so the array reads as
    //    `[[1,"group",3],[5,"faction","BLU_F"],[8,"missionName","OpTest"]]`, giving
    //    (1, group, 3), (5, faction, BLU_F) and (8, missionName, OpTest) once
    //    `trimWhitespaceColon` strips the quotes.
    // 2. The action is not "beta", so the stream starts
    //    `piwik.php?idsite=2&rec=1&url="radio.task-force.ru"` and the host is `PIWIK_URL`,
    //    `nkey.piwik.pro` (`reference/tfar_1_0/ts/src/common.hpp`).
    // 3. It appends `&action_name=start`, `&rand=41` (the fixed value here) and
    //    `&uid=76561198012345678`.
    // 4. `&cvar={` then, comma-separated, `"<id>":["<name>","<value>"]` per variable, then `}`:
    //    `&cvar={"1":["group","3"],"5":["faction","BLU_F"],"8":["missionName","OpTest"]}`.
    // 5. The request line cannot carry `"`, so each one is sent as `%22`; `{`, `}`, `[`, `]`, `:`
    //    and `,` are legal in a query and stay as they are. The path gains its leading `/`.
    let expected = concat!(
        "http://nkey.piwik.pro/piwik.php?idsite=2&rec=1&url=%22radio.task-force.ru%22",
        "&action_name=start&rand=41&uid=76561198012345678",
        "&cvar={%221%22:[%22group%22,%223%22],%225%22:[%22faction%22,%22BLU_F%22],",
        "%228%22:[%22missionName%22,%22OpTest%22]}",
    );
    let url = build_track_request_1_0_with_rand("start", SAMPLE_UID, SAMPLE_CUSTOM_VARS, 41)
        .expect("the URL builds");
    assert_eq!(url.as_str(), expected);
}

#[test]
fn the_beta_action_goes_to_the_beta_host_and_names_it_as_the_site() {
    let url = build_track_request_1_0_with_rand("beta", SAMPLE_UID, r#"[[1,"a","b"]]"#, 7)
        .expect("the URL builds");
    assert_eq!(
        url.as_str(),
        concat!(
            "http://piwik.dedmen.de/piwik.php?idsite=2&rec=1&url=%22piwik.dedmen.de%22",
            "&action_name=beta&rand=7&uid=76561198012345678&cvar={%221%22:[%22a%22,%22b%22]}",
        )
    );
}

#[test]
fn the_continue_action_uses_the_release_host() {
    let url = build_track_request_1_0_with_rand("continue", "1", r#"[[1,"a","b"]]"#, 0)
        .expect("the URL builds");
    assert_eq!(url.host_str(), Some(PIWIK_HOST));
    assert_eq!(url.scheme(), "http");
    assert_eq!(url.port_or_known_default(), Some(80));
}

#[test]
fn no_custom_variable_means_no_request() {
    let result = build_track_request_1_0("start", SAMPLE_UID, "[]");
    assert!(
        matches!(&result, Err(Error::NoCustomVariables { text }) if text == "[]"),
        "{result:?}"
    );
}

#[test]
fn the_random_parameter_stays_in_the_msvc_rand_range() {
    for _ in 0..64 {
        let url = build_track_request_1_0("start", SAMPLE_UID, r#"[[1,"a","b"]]"#)
            .expect("the URL builds");
        let rand: u32 = url
            .query_pairs()
            .find(|(key, _)| key == "rand")
            .map(|(_, value)| value.parse().expect("rand is a number"))
            .expect("the query carries rand");
        assert!(rand < 32_768, "rand {rand}");
    }
}

#[test]
fn a_hash_in_a_value_is_encoded_instead_of_starting_a_fragment() {
    let url = build_track_request_1_0_with_rand("start", "u", r#"[[1,"a","b#c"]]"#, 1)
        .expect("the URL builds");
    assert_eq!(url.fragment(), None);
    assert!(url.as_str().ends_with("%22b%23c%22]}"), "{url}");
}
