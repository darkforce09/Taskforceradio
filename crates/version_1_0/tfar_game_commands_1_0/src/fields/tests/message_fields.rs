//! Unit tests for `MessageFields` and the field split.

use super::*;

#[test]
fn the_split_keeps_every_field_as_helpers_split_does() {
    // reference/tfar_1_0/ts/src/helpers.cpp:245-262: the loop runs while lastPos < length + 1,
    // so a trailing separator gives a trailing empty field and an empty text one empty field.
    let table: [(&str, &[&str]); 6] = [
        ("", &[""]),
        ("a", &["a"]),
        ("a\t", &["a", ""]),
        ("a\t\tb", &["a", "", "b"]),
        ("\ta", &["", "a"]),
        ("\t", &["", ""]),
    ];
    for (input, expected) in table {
        assert_eq!(
            split_keeping_every_field(input, '\t'),
            expected,
            "{input:?}"
        );
    }
}

fn fields(tokens: &[&'static str]) -> MessageFields<'static> {
    MessageFields::new(CommandName::Position, tokens.to_vec())
}

#[test]
fn a_missing_token_is_a_token_count_error() {
    let fields = fields(&["POS", "nick"]);
    assert_eq!(fields.text(1), Ok("nick"));
    assert_eq!(
        fields.text(5),
        Err(Error::WrongTokenCount {
            command: CommandName::Position,
            minimum: 6,
            maximum: usize::MAX,
            found: 2
        })
    );
}

#[test]
fn count_checks_name_the_accepted_counts() {
    let fields = fields(&["POS", "a", "b"]);
    assert!(fields.require_count(3).is_ok());
    assert!(fields.require_at_least(2).is_ok());
    assert!(fields.require_between(2, 4).is_ok());
    let error = fields.require_count(14).unwrap_err();
    assert_eq!(
        error.to_string(),
        "POS needs exactly 14 tokens, the name included, but has 3"
    );
    assert_eq!(
        fields.require_between(5, 6).unwrap_err().to_string(),
        "POS needs 5 to 6 tokens, the name included, but has 3"
    );
}

#[test]
fn a_malformed_number_names_the_field() {
    let fields = fields(&["POS", "ten"]);
    assert_eq!(
        fields.number(1, "voice volume"),
        Err(Error::Malformed {
            command: CommandName::Position,
            field: "voice volume",
            value: "ten".to_owned()
        })
    );
}

#[test]
fn numbers_round_and_nicknames_sanitize() {
    let fields = fields(&["POS", "2.5", " nick "]);
    assert_eq!(fields.integer(1, "n"), Ok(3));
    assert_eq!(fields.nickname(2), Ok("_nick_".to_owned()));
    assert_eq!(fields.after_name(), ["2.5", " nick "]);
}
