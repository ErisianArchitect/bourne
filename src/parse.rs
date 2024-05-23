// Why did the digital archaeologist get excited about old software?
// Because they loved discovering ancient "bits" of history!

use std::str::FromStr;

use crate::{error::ParseError, Value, ValueMap};

pub type ParseResult<T> = Result<T, ParseError>;

fn hex_value(digit: char) -> Option<u16> {
    if digit >= '0' && digit <= '9' {
        Some(digit as u16 - '0' as u16)
    } else if digit >= 'a' && digit <= 'f' {
        Some(digit as u16 - 'a' as u16 + 10)
    } else if digit >= 'A' && digit <= 'F' {
        Some(digit as u16 - 'A' as u16 + 10)
    } else {
        None
    }
}

/// Unescape a string.
pub fn unescape_string<S: AsRef<str>>(s: S) -> ParseResult<String> {
    let s = s.as_ref();
    let mut buffer = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            buffer.push(c);
            continue;
        }
        buffer.push(match chars.next() {
            Some(single @ ('/' | '\\' | '"')) => single,
            Some('f') => '\u{c}',
            Some('b') => '\u{8}',
            Some('n') => '\n',
            Some('r') => '\r',
            Some('t') => '\t',
            Some('u') => {
                // Read 4 hex-digits
                let mut hex: u16 = 0;
                for i in 0..4 {
                    let Some(digit) = chars.next() else {
                        return Err(ParseError::UnexpectedEOF);
                    };
                    let Some(value) = hex_value(digit) else {
                        return Err(ParseError::InvalidHex);
                    };
                    hex |= value;
                    // Do not shift if it's the last cycle.
                    if i < 3 {
                        hex <<= 4;
                    }
                }
                let Some(res) = char::from_u32(hex as u32) else {
                    return Err(ParseError::InvalidEscapeSequence);
                };
                res
            }
            Some(other) => other,
            None => return Err(ParseError::UnexpectedEOF),
        });
    }
    Ok(buffer)
}

#[derive(Debug, Clone, Copy)]
struct Parser<'a> {
    source: &'a str,
    index: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            index: 0,
        }
    }

    fn is_eof(&self) -> bool {
        self.index >= self.source.len()
    }

    fn peek(&self) -> Option<u8> {
        if self.index < self.source.len() {
            Some(self.source.as_bytes()[self.index])
        } else {
            None
        }
    }

    fn check_next<F: Fn(&u8) -> bool>(&mut self, step: bool, predicate: F) -> Option<bool> {
        let res = predicate(&self.peek()?);
        if step && res {
            self.index += 1;
        }
        Some(res)
    }

    fn indexed_next(&mut self) -> Option<(usize, u8)> {
        if self.index < self.source.len() {
            let res = Some((self.index, self.source.as_bytes()[self.index]));
            self.index += 1;
            res
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<u8> {
        if self.index < self.source.len() {
            let res = Some(self.source.as_bytes()[self.index]);
            self.index += 1;
            res
        } else {
            None
        }
    }

    fn advance(&mut self, step: usize) {
        self.index += step;
    }

    fn fork(&self) -> Self {
        self.clone()
    }

    /// `fork` must have been created using `fork()` method.
    fn step_to(&mut self, fork: &Self) -> ParseResult<()> {
        if self.source == fork.source {
            self.index = fork.index;
            Ok(())
        } else {
            Err(ParseError::StepToInvalidFork)
        }
    }

    fn rewind(&mut self) {
        self.index = self.index.checked_sub(1).unwrap_or(0);
    }

    fn matches<S: AsRef<str>>(&mut self, s: S) -> bool {
        let s = s.as_ref();
        if self.index + s.len() <= self.source.len() {
            s == &self.source[self.index..self.index + s.len()]
        } else {
            false
        }
    }

    fn eat_whitespace(&mut self) {
        while let Some(true) = self.check_next(true, u8::is_ascii_whitespace) {}
    }

    fn parse_null(&mut self) -> ParseResult<Value> {
        if self.matches("null") {
            self.advance(4);
            Ok(Value::Null)
        } else {
            Err(ParseError::InvalidCharacter(self.index))
        }
    }

    fn parse_boolean(&mut self) -> ParseResult<bool> {
        if self.matches("true") {
            self.advance(4);
            Ok(true)
        } else if self.matches("false") {
            self.advance(5);
            Ok(false)
        } else {
            Err(ParseError::InvalidCharacter(self.index))
        }
    }

    fn parse_number(&mut self) -> ParseResult<f64> {
        let mut fork = self.fork();
        // Valid characters that can follow a number: '}', ']', ',', and whitespace.
        let mut found_e = false;
        let mut found_dot = false;
        let start = fork.index;
        if let Some(b'-' | b'+') = fork.peek() {
            fork.next();
        }
        while let Some((index, next)) = fork.indexed_next() {
            match next {
                b'0'..=b'9' => {}
                b'.' if !found_dot && !found_e => found_dot = true,
                b'e' | b'E' if !found_e => {
                    found_e = true;
                    if matches!(fork.peek(), Some(b'+' | b'-')) {
                        fork.advance(1);
                    }
                },
                b'}' | b']' | b',' => {
                    fork.rewind();
                    break
                },
                ws if ws.is_ascii_whitespace() => {
                    fork.rewind();
                    break
                },
                _ => return Err(ParseError::InvalidCharacter(index)),
            }
        }
        let value = fork.source[start..fork.index].parse::<f64>()?;
        self.step_to(&fork)?;
        Ok(value)
    }

    fn parse_string(&mut self) -> ParseResult<String> {
        let mut fork = self.fork();
        match fork.peek() {
            Some(b'"') => { fork.next(); }
            Some(_) => { return Err(ParseError::InvalidCharacter(fork.index)); }
            None => { return Err(ParseError::UnexpectedEOF); }
        }
        let start = fork.index;
        let string = loop {
            let Some((index, next)) = fork.indexed_next() else {
                return Err(ParseError::UnexpectedEOFWhileParsingString(start));
            };
            match next {
                // Strings should not contain new-lines.
                b'\n' | b'\r' => { return Err(ParseError::LineBreakWhileParsingString(index)); }
                b'"' => break unescape_string(&fork.source[start..index])?,
                b'\\' => { fork.advance(1); }
                _ => {}
            }
        };
        self.step_to(&fork)?;
        Ok(string)
    }

    fn parse_array(&mut self) -> ParseResult<Vec<Value>> {
        let mut fork = self.fork();
        match fork.indexed_next() {
            Some((_, b'[')) => (),
            Some((index, _)) => return Err(ParseError::InvalidCharacter(index)),
            None => return Err(ParseError::UnexpectedEOF),
        }
        let mut array = Vec::new();
        loop {
            fork.eat_whitespace();
            match fork.peek() {
                Some(b']') => {
                    fork.advance(1);
                    break;
                }
                Some(_) => {
                    array.push(fork.parse_value()?);
                    fork.eat_whitespace();
                    match fork.indexed_next() {
                        Some((_, b']')) => break,
                        Some((_, b',')) => continue,
                        Some((index, _)) => return Err(ParseError::InvalidCharacter(index)),
                        None => return Err(ParseError::UnexpectedEOF),
                    }
                }
                None => return Err(ParseError::UnexpectedEOF),
            }
        }
        self.step_to(&fork)?;
        Ok(array)
    }

    fn parse_object(&mut self) -> ParseResult<ValueMap> {
        let mut fork = self.fork();
        match fork.indexed_next() {
            Some((_, b'{')) => (),
            Some((index, _)) => return Err(ParseError::InvalidCharacter(index)),
            None => return Err(ParseError::UnexpectedEOF),
        }
        let mut map = ValueMap::new();
        loop {
            fork.eat_whitespace();
            match fork.peek() {
                Some(b'"') => {
                    let key = fork.parse_string()?;
                    fork.eat_whitespace();
                    match fork.indexed_next() {
                        Some((_, b':')) => (),
                        Some((index, _)) => return Err(ParseError::InvalidCharacter(index)),
                        None => return Err(ParseError::UnexpectedEOF),
                    }
                    fork.eat_whitespace();
                    let value = fork.parse_value()?;
                    map.insert(key, value);
                    fork.eat_whitespace();
                    match fork.indexed_next() {
                        Some((_, b',')) => continue,
                        Some((_, b'}')) => break,
                        Some((index, _)) => return Err(ParseError::InvalidCharacter(index)),
                        None => return Err(ParseError::UnexpectedEOF),
                    }
                }
                Some(b'}') => {
                    fork.next();
                    break;
                }
                Some(_) => return Err(ParseError::InvalidCharacter(fork.index)),
                None => return Err(ParseError::UnexpectedEOF),
            }
        }
        self.step_to(&fork)?;
        Ok(map)
    }

    fn parse_value(&mut self) -> ParseResult<Value> {
        let mut fork = self.fork();
        let value = match fork.peek() {
            Some(b't' | b'f') => Value::Boolean(fork.parse_boolean()?),
            Some(b'n') => fork.parse_null()?,
            Some(b'"') => Value::String(fork.parse_string()?),
            Some(b'{') => Value::Object(fork.parse_object()?),
            Some(b'[') => Value::Array(fork.parse_array()?),
            Some(_) => Value::Number(fork.parse_number()?),
            None => return Err(ParseError::UnexpectedEOF),
        };
        self.step_to(&fork)?;
        Ok(value)
    }
}

impl FromStr for Value {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = Parser::new(s);
        parser.eat_whitespace();
        let res = parser.parse_value()?;
        parser.eat_whitespace();
        if !parser.is_eof() {
            Err(ParseError::InvalidCharacter(parser.index))
        } else {
            Ok(res)
        }
    }
}

#[test]
fn quick() {
    let s = r#"{"number":-0.31415e1,"bool":[false,true,false],"null":null,"string":"Hello, world!"}"#;
    let value = Value::from_str(s);
    println!("{value:#?}");
    // match parse_number(&mut parser) {
    //     Ok(number) => {
    //         println!("Number: {number}");
    //     }
    //     Err(err) => {
    //         println!("{err}");
    //     }
    // }
}

// fn parse_value(parser: &mut Parser<'_>)