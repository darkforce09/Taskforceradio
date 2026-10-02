//! Unit tests for `parse_bool`.

use super::*;

#[test]
fn true_and_false_parse_in_any_letter_case() {
    for text in ["true", "True", "TRUE", " true "] {
        assert!(parse_bool(text).unwrap(), "text {text:?}");
    }
    for text in ["false", "False", "FALSE", " false"] {
        assert!(!parse_bool(text).unwrap(), "text {text:?}");
    }
}

#[test]
fn any_other_value_is_an_error() {
    for text in ["", "1", "0", "yes", "truee", "t", "any"] {
        assert_eq!(
            parse_bool(text),
            Err(Error::NotABoolean {
                value: text.to_owned()
            }),
            "text {text:?}"
        );
    }
}
