use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Attempted to step to an invalid fork. Fork must have been created from the parser that you wish to join to.")]
    StepToInvalidFork,
    #[error("Invalid character at index {0}.")]
    InvalidCharacter(usize),
    #[error("Unexpected end of stream.")]
    UnexpectedEOF,
    #[error("Unexpected end of stream while parsing string; Start Index: {0}")]
    UnexpectedEOFWhileParsingString(usize),
    #[error("Line Break while parsing string. Index: {0}")]
    LineBreakWhileParsingString(usize),
    #[error("Parse Integer Error: {0}")]
    ParseIntError(#[from]std::num::ParseIntError),
    #[error("Parse Float Error: {0}")]
    ParseFloatError(#[from]std::num::ParseFloatError),
    #[error("Invalid escape sequence.")]
    InvalidEscapeSequence,
    #[error("Invalid Unicode Character.")]
    InvalidUnicodeCharacter,
    #[error("Invalid Hex.")]
    InvalidHex,
}