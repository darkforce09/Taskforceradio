//! Unit tests for the position and string array parsers.

use super::*;

#[test]
fn a_position_array_parses_into_a_position() {
    let position = parse_position_array("[1234.5,6789,12.25]").unwrap();
    assert_eq!(position.to_array(), [1234.5, 6789.0, 12.25]);
}

#[test]
fn a_position_array_allows_spaces_and_scientific_notation() {
    let position = parse_position_array(" [ 1.5e+004 , -2 ,3.25e-001 ] ").unwrap();
    assert_eq!(position.to_array(), [15_000.0, -2.0, 0.325]);
}

#[test]
fn a_position_array_needs_three_elements() {
    for (text, found) in [("[]", 0), ("[1,2]", 2), ("[1,2,3,4]", 4)] {
        assert_eq!(
            parse_position_array(text),
            Err(Error::WrongElementCount {
                expected: 3,
                found,
                value: text.to_owned()
            }),
            "text {text:?}"
        );
    }
}

#[test]
fn a_position_array_needs_brackets() {
    for text in ["1,2,3", "[1,2,3", "1,2,3]", ""] {
        assert!(matches!(
            parse_position_array(text),
            Err(Error::NotAnArray { .. })
        ));
    }
}

#[test]
fn a_malformed_coordinate_is_an_error() {
    assert!(matches!(
        parse_position_array("[1,x,3]"),
        Err(Error::MalformedNumber { .. })
    ));
    assert!(matches!(
        parse_position_array("[1,,3]"),
        Err(Error::MalformedNumber { .. })
    ));
}

#[test]
fn an_empty_array_is_no_position() {
    assert_eq!(parse_optional_position("[]").unwrap(), None);
    assert_eq!(parse_optional_position(" [ ] ").unwrap(), None);
}

#[test]
fn an_optional_position_parses_like_a_position_array() {
    let position = parse_optional_position("[1,2,3]").unwrap().unwrap();
    assert_eq!(position.to_array(), [1.0, 2.0, 3.0]);
    assert!(parse_optional_position("[1,2]").is_err());
    assert!(parse_optional_position("nothing").is_err());
}

#[test]
fn a_string_array_parses_into_its_strings() {
    assert_eq!(
        parse_string_array(r#"["a","b"]"#).unwrap(),
        ["a".to_owned(), "b".to_owned()]
    );
    assert_eq!(
        parse_string_array(r#"["87.6|7|0"]"#).unwrap(),
        ["87.6|7|0".to_owned()]
    );
}

#[test]
fn an_empty_string_array_gives_an_empty_list() {
    assert_eq!(parse_string_array("[]").unwrap(), Vec::<String>::new());
    assert_eq!(parse_string_array(" [  ] ").unwrap(), Vec::<String>::new());
}

#[test]
fn a_comma_inside_quotes_does_not_split() {
    assert_eq!(
        parse_string_array(r#"["a,b","c"]"#).unwrap(),
        ["a,b".to_owned(), "c".to_owned()]
    );
}

#[test]
fn a_doubled_quote_is_one_quote() {
    assert_eq!(
        parse_string_array(r#"["say ""hi""",""]"#).unwrap(),
        [r#"say "hi""#.to_owned(), String::new()]
    );
}

#[test]
fn spaces_around_elements_and_commas_are_allowed() {
    assert_eq!(
        parse_string_array(r#"[ "a" , "b" ]"#).unwrap(),
        ["a".to_owned(), "b".to_owned()]
    );
}

#[test]
fn an_unquoted_element_is_an_error() {
    assert_eq!(
        parse_string_array(r#"["a",b]"#),
        Err(Error::UnquotedElement {
            index: 1,
            value: r#"["a",b]"#.to_owned()
        })
    );
    assert!(matches!(
        parse_string_array(r#"["a"x]"#),
        Err(Error::UnquotedElement { index: 0, .. })
    ));
    assert!(matches!(
        parse_string_array(r#"["a",]"#),
        Err(Error::UnquotedElement { index: 1, .. })
    ));
}

#[test]
fn an_unterminated_string_is_an_error() {
    assert!(matches!(
        parse_string_array(r#"["a","b]"#),
        Err(Error::UnterminatedString { .. })
    ));
}

#[test]
fn a_string_array_needs_brackets() {
    assert!(matches!(
        parse_string_array(r#""a","b""#),
        Err(Error::NotAnArray { .. })
    ));
}
