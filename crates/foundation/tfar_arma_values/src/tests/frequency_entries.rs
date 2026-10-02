//! Unit tests for `parse_frequency_entries`.

use super::*;
use crate::error::Error;

fn entry(
    frequency: &str,
    volume: f64,
    stereo_mode: StereoMode,
    classname: Option<&str>,
) -> FrequencyEntry {
    FrequencyEntry {
        frequency: frequency.to_owned(),
        volume,
        stereo_mode,
        radio_classname: classname.map(str::to_owned),
    }
}

#[test]
fn three_part_entries_parse_for_both_shapes() {
    let text = r#"["87.6|7|0","45.5_2|5|1"]"#;
    let expected = vec![
        entry("87.6", 7.0, StereoMode::Both, None),
        entry("45.5_2", 5.0, StereoMode::Left, None),
    ];
    for shape in [
        FrequencyEntryShape::ThreeParts,
        FrequencyEntryShape::ThreeOrFourParts,
    ] {
        assert_eq!(parse_frequency_entries(text, shape).unwrap(), expected);
    }
}

#[test]
fn four_part_entries_carry_the_classname_in_the_one_zero_shape() {
    let entries = parse_frequency_entries(
        r#"["30|7|2|tf_rt1523g","40|3|1"]"#,
        FrequencyEntryShape::ThreeOrFourParts,
    )
    .unwrap();
    assert_eq!(
        entries,
        [
            entry("30", 7.0, StereoMode::Right, Some("tf_rt1523g")),
            entry("40", 3.0, StereoMode::Left, None),
        ]
    );
}

#[test]
fn four_part_entries_are_skipped_in_the_zero_nine_shape() {
    let entries = parse_frequency_entries(
        r#"["30|7|2|tf_rt1523g","40|3|1"]"#,
        FrequencyEntryShape::ThreeParts,
    )
    .unwrap();
    assert_eq!(entries, [entry("40", 3.0, StereoMode::Left, None)]);
}

#[test]
fn entries_of_the_wrong_part_count_are_skipped() {
    let text = r#"["30|7","","a|1|0|b|c","50|2|0"]"#;
    for shape in [
        FrequencyEntryShape::ThreeParts,
        FrequencyEntryShape::ThreeOrFourParts,
    ] {
        assert_eq!(
            parse_frequency_entries(text, shape).unwrap(),
            [entry("50", 2.0, StereoMode::Both, None)]
        );
    }
}

#[test]
fn a_trailing_empty_part_follows_each_versions_split() {
    // getline drops the empty fourth part, leaving three parts; the 1.0 split keeps it, and an
    // empty classname reads as none.
    let text = r#"["30|7|1|"]"#;
    assert_eq!(
        parse_frequency_entries(text, FrequencyEntryShape::ThreeParts).unwrap(),
        [entry("30", 7.0, StereoMode::Left, None)]
    );
    assert_eq!(
        parse_frequency_entries(text, FrequencyEntryShape::ThreeOrFourParts).unwrap(),
        [entry("30", 7.0, StereoMode::Left, None)]
    );
    // "30|7|" is two getline parts but three 1.0 parts with an empty stereo mode.
    assert_eq!(
        parse_frequency_entries(r#"["30|7|"]"#, FrequencyEntryShape::ThreeParts).unwrap(),
        Vec::<FrequencyEntry>::new()
    );
    assert!(matches!(
        parse_frequency_entries(r#"["30|7|"]"#, FrequencyEntryShape::ThreeOrFourParts),
        Err(Error::MalformedNumber { .. })
    ));
}

#[test]
fn volume_and_stereo_mode_are_rounded() {
    let entries =
        parse_frequency_entries(r#"["30|6.5|1.6"]"#, FrequencyEntryShape::ThreeParts).unwrap();
    assert_eq!(entries, [entry("30", 7.0, StereoMode::Right, None)]);
}

#[test]
fn an_unknown_stereo_mode_reads_as_both() {
    let entries =
        parse_frequency_entries(r#"["30|7|9"]"#, FrequencyEntryShape::ThreeParts).unwrap();
    assert_eq!(entries, [entry("30", 7.0, StereoMode::Both, None)]);
}

#[test]
fn an_empty_list_gives_no_entries() {
    assert_eq!(
        parse_frequency_entries("[]", FrequencyEntryShape::ThreeOrFourParts).unwrap(),
        Vec::<FrequencyEntry>::new()
    );
}

#[test]
fn a_malformed_volume_is_an_error_for_the_list() {
    assert_eq!(
        parse_frequency_entries(r#"["30|loud|0"]"#, FrequencyEntryShape::ThreeParts),
        Err(Error::MalformedNumber {
            value: "loud".to_owned()
        })
    );
}

#[test]
fn a_list_that_is_not_a_string_array_is_an_error() {
    assert!(matches!(
        parse_frequency_entries("87.6|7|0", FrequencyEntryShape::ThreeParts),
        Err(Error::NotAnArray { .. })
    ));
}
