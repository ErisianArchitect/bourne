use thiserror::Error;

#[derive(Debug, Error)]
pub enum BourneError {
    #[error("Parse Error: {0}")]
    ParseError(#[from]ParseError),
    #[error("Attempted to push to a Value that was not an array.")]
    PushToNonArray,
    #[error("Attempted to insert into a Value that was not an object.")]
    InsertToNonObject,
}

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