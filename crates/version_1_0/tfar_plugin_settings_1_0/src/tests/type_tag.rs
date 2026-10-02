//! Unit tests for `TypeTag`: the tag names and how each reads its value.

use super::*;

#[test]
fn the_tags_match_exactly() {
    // reference/tfar_1_0/ts/src/CommandProcessor.cpp:404 and :406.
    assert_eq!(TypeTag::from_wire("BOOL"), TypeTag::Bool);
    assert_eq!(TypeTag::from_wire("SCALAR"), TypeTag::Scalar);
    assert_eq!(TypeTag::from_wire("STRING"), TypeTag::Text);
    assert_eq!(TypeTag::from_wire("bool"), TypeTag::Text);
    assert_eq!(TypeTag::from_wire(""), TypeTag::Text);
}

#[test]
fn a_bool_value_is_true_only_for_true_in_lower_or_upper_case() {
    // reference/tfar_1_0/ts/src/CommandProcessor.cpp:405.
    assert_eq!(
        TypeTag::Bool.read_value("true"),
        Ok(SettingValue::Bool(true))
    );
    assert_eq!(
        TypeTag::Bool.read_value("TRUE"),
        Ok(SettingValue::Bool(true))
    );
    assert_eq!(
        TypeTag::Bool.read_value("True"),
        Ok(SettingValue::Bool(false))
    );
    assert_eq!(TypeTag::Bool.read_value("1"), Ok(SettingValue::Bool(false)));
    assert_eq!(
        TypeTag::Bool.read_value("false"),
        Ok(SettingValue::Bool(false))
    );
}

#[test]
fn a_scalar_value_is_a_number() {
    assert_eq!(
        TypeTag::Scalar.read_value("339"),
        Ok(SettingValue::Number(339.0))
    );
    assert_eq!(
        TypeTag::Scalar.read_value("0.3"),
        Ok(SettingValue::Number(0.3))
    );
    assert_eq!(
        TypeTag::Scalar.read_value("1.5e+006"),
        Ok(SettingValue::Number(1.5e6))
    );
}

#[test]
fn a_malformed_scalar_is_an_error() {
    assert_eq!(
        TypeTag::Scalar.read_value("fast"),
        Err(Error::MalformedNumber {
            value: "fast".to_owned()
        })
    );
    assert!(TypeTag::Scalar.read_value("").is_err());
    assert!(TypeTag::Scalar.read_value("inf").is_err());
}

#[test]
fn any_other_tag_reads_the_value_as_text() {
    assert_eq!(
        TypeTag::Text.read_value("TaskForceRadio"),
        Ok(SettingValue::Text("TaskForceRadio".to_owned()))
    );
}
