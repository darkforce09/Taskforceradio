//! Unit tests for `SettingValue`: the cross-kind conversions of `settingValue::operator=`.

use super::*;

#[test]
fn a_boolean_setting_takes_a_number_as_greater_than_zero() {
    // reference/tfar_1_0/ts/src/settings.hpp:90.
    assert_eq!(SettingValue::Number(0.5).into_bool(), Some(true));
    assert_eq!(SettingValue::Number(0.0).into_bool(), Some(false));
    assert_eq!(SettingValue::Number(-1.0).into_bool(), Some(false));
    assert_eq!(SettingValue::Bool(true).into_bool(), Some(true));
}

#[test]
fn a_number_setting_takes_a_boolean_as_one_or_zero() {
    // reference/tfar_1_0/ts/src/settings.hpp:83.
    assert_eq!(SettingValue::Bool(true).into_number(), Some(1.0));
    assert_eq!(SettingValue::Bool(false).into_number(), Some(0.0));
    assert_eq!(SettingValue::Number(339.0).into_number(), Some(339.0));
}

#[test]
fn a_text_setting_takes_a_boolean_as_true_or_false() {
    // reference/tfar_1_0/ts/src/settings.hpp:84.
    assert_eq!(SettingValue::Bool(true).into_text(), "true");
    assert_eq!(SettingValue::Bool(false).into_text(), "false");
}

#[test]
fn a_text_setting_takes_a_number_as_the_six_decimal_text_of_std_to_string() {
    // std::to_string(float), reference/tfar_1_0/ts/src/settings.hpp:92.
    assert_eq!(SettingValue::Number(339.0).into_text(), "339.000000");
    assert_eq!(SettingValue::Number(0.5).into_text(), "0.500000");
    assert_eq!(SettingValue::Number(-2.25).into_text(), "-2.250000");
}

#[test]
fn a_text_never_becomes_a_boolean_or_a_number() {
    assert_eq!(SettingValue::Text("true".to_owned()).into_bool(), None);
    assert_eq!(SettingValue::Text("1".to_owned()).into_number(), None);
}

#[test]
fn converted_to_follows_the_target_kind() {
    assert_eq!(
        SettingValue::Number(2.0).converted_to(SettingKind::Bool),
        Some(SettingValue::Bool(true))
    );
    assert_eq!(
        SettingValue::Bool(true).converted_to(SettingKind::Text),
        Some(SettingValue::Text("true".to_owned()))
    );
    assert_eq!(
        SettingValue::Text(String::new()).converted_to(SettingKind::Number),
        None
    );
}
