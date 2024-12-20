use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    /// Invalid character found in the JSON text while parsing.
    #[error("Invalid character at index {0}.")]
    InvalidCharacter(usize),
    /// Unexpectedly reached the end of the stream.
    #[error("Unexpected end of stream.")]
    UnexpectedEOF,
    /// Unexpectedly reached the end of the stream while parsing a [String].
    #[error("Unexpected end of stream while parsing string; Start Index: {0}")]
    UnexpectedEOFWhileParsingString(usize),
    /// Line break was found while parsing [String]. End quotes must be on the same line.
    #[error("Line Break while parsing string. End quote must be on same line. Index: {0}")]
    LineBreakWhileParsingString(usize),
    /// Error parsing integer.
    #[error("Parse Int Error: {0}")]
    ParseIntError(#[from]std::num::ParseIntError),
    /// Error parsing floating point number.
    #[error("Parse Float Error: {0}")]
    ParseFloatError(#[from]std::num::ParseFloatError),
    /// Invalid escape sequence in [String].
    #[error("Invalid escape sequence.")]
    InvalidEscapeSequence,
    /// Invalid hexadecimal value.
    #[error("Invalid Hex.")]
    InvalidHex,
}