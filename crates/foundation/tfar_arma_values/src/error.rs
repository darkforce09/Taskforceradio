//! The error type of `tfar_arma_values`.

/// Every failure `tfar_arma_values` reports. Each variant carries the text that failed, so a log
/// line can show what the game sent.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    /// The text is not a finite Arma number.
    #[error("`{value}` is not a finite number")]
    MalformedNumber {
        /// The text that failed to parse.
        value: String,
    },
    /// The number rounds to an integer outside the range of `i32`.
    #[error("`{value}` rounds to an integer outside the 32-bit range")]
    IntegerOutOfRange {
        /// The text that failed to parse.
        value: String,
    },
    /// The text is neither `true` nor `false`, in any letter case.
    #[error("`{value}` is neither `true` nor `false`")]
    NotABoolean {
        /// The text that failed to parse.
        value: String,
    },
    /// The text is not enclosed in `[` and `]`.
    #[error("`{value}` is not an Arma array")]
    NotAnArray {
        /// The text that failed to parse.
        value: String,
    },
    /// The array holds a different number of elements than the shape needs.
    #[error("`{value}` holds {found} elements where {expected} are needed")]
    WrongElementCount {
        /// The number of elements the shape needs.
        expected: usize,
        /// The number of elements the array holds.
        found: usize,
        /// The text that failed to parse.
        value: String,
    },
    /// An element of a string array is not a quoted string, or text follows its closing quote.
    #[error("element {index} of `{value}` is not a quoted string")]
    UnquotedElement {
        /// The zero-based position of the element.
        index: usize,
        /// The text that failed to parse.
        value: String,
    },
    /// A quoted string in an array has no closing quote.
    #[error("`{value}` holds a string with no closing quote")]
    UnterminatedString {
        /// The text that failed to parse.
        value: String,
    },
    /// The coordinates parsed but do not form a position.
    #[error(transparent)]
    Geometry(#[from] tfar_geometry::Error),
}

/// The result of a fallible `tfar_arma_values` operation.
pub type Result<T> = core::result::Result<T, Error>;
