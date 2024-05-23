#![allow(unused)]

use std::fmt::Write;
use std::str::FromStr;

use crate::error::*;
use crate::{
    Value,
    ValueMap,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Indent {
    None,
    Spaces(u8),
    Tabs(u8),
}

impl std::fmt::Display for Indent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const SPACES: &'static str = "                ";
        const TABS: &'static str = "\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t";
        fn write_all(f: &mut std::fmt::Formatter<'_>, text: &'static str, count: u8) -> std::fmt::Result {
            let mut count = count as usize;
            while count > 0 {
                let len = count.min(SPACES.len());
                write!(f, "{}", &text[..len])?;
                count -= len;
            }
            Ok(())
        }
        match self {
            Self::None => {}
            &Self::Spaces(count) => write_all(f, SPACES, count)?,
            &Self::Tabs(count) => write_all(f, TABS, count)?,
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct JsonFormatter {
    /// All on the same line.
    sameline: bool,
    /// No spaces between elements or around colons.
    no_spacing: bool,
    /// The [Indent] to use. This is ignored if `sameline` is true.
    indent: Indent,
    /// Indent level. Only modify this if you know what you're doing.
    indent_level: u32,
}

impl JsonFormatter {
    pub fn new(sameline: bool, no_spacing: bool, indent: Indent) -> Self {
        Self::new_indented(0, sameline, no_spacing, indent)
    }

    pub fn new_indented(indent_level: u32, sameline: bool, no_spacing: bool, indent: Indent) -> Self {
        Self {
            sameline,
            no_spacing,
            indent,
            indent_level: indent_level,
        }
    }

    /// Creates a copy of self where the indent level is incremented by 1.
    pub fn indent(&self) -> Self {
        Self {
            sameline: self.sameline,
            no_spacing: self.no_spacing,
            indent: self.indent,
            indent_level: self.indent_level + 1,
        }
    }

    /// Writes the indentation to a writer.
    pub fn write_indent<W: Write>(&self, writer: &mut W) -> std::fmt::Result {
        for _ in 0..self.indent_level {
            write!(writer, "{}", self.indent)?;
        }
        Ok(())
    }

    pub fn write_separator<W: Write>(&self, writer: &mut W) -> std::fmt::Result {
        write!(writer, ",")?;
        if !self.sameline {
            write!(writer, "\n")?;
        // Are double-negatives allowed in programming? There's not no spacing here.
        } else if !self.no_spacing {
            write!(writer, " ")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for JsonFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.sameline {
            self.write_indent(f)?;
        }
        Ok(())
    }
}

impl Default for Indent {
    /// Indent::Spaces(4)
    fn default() -> Self {
        Self::Spaces(4)
    }
}

fn hex_char(value: u16, slot: usize) -> char {
    const HEX_CHARS: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'];
    let shift = slot * 4;
    let index = ((value & (0xf << shift)) >> shift) as usize;
    HEX_CHARS[index]
}

/// Measures the length of a string after being escaped.
pub fn measure_escaped_string<S: AsRef<str>>(s: S) -> usize {
    s.as_ref().chars().map(|c| {
        match c {
            '/' => 2,
            '\\' => 2,
            '"' => 2,
            '\u{c}' => 2,
            '\u{8}' => 2,
            '\n' => 2,
            '\r' => 2,
            '\t' => 2,
            '\u{0}'..='\u{1f}' => 6,
            _ => c.len_utf8(),
        }
    }).sum()
}

/// Escapes a string.
pub fn escape_string<S: AsRef<str>>(s: S) -> String {
    let mut buffer = String::with_capacity(measure_escaped_string(s.as_ref()));
    s.as_ref().chars().for_each(|c| {
        match c {
            '/' => buffer.push_str("\\/"),
            '\\' => buffer.push_str("\\\\"),
            '"' => buffer.push_str("\\\""),
            '\u{c}' => buffer.push_str("\\f"),
            '\u{8}' => buffer.push_str("\\b"),
            '\n' => buffer.push_str("\\n"),
            '\r' => buffer.push_str("\\r"),
            '\t' => buffer.push_str("\\t"),
            '\u{0}'..='\u{1f}' => {
                let hex = c as u16;
                buffer.push_str("\\u00");
                for i in (0..2).rev() {
                    buffer.push(hex_char(hex, i));
                }
            }
            _ => buffer.push(c),
        }
    });
    buffer
}

fn write_escaped_string<W: Write, S: AsRef<str>>(writer: &mut W, s: S) -> std::fmt::Result {
    s.as_ref().chars().try_for_each(|c| {
        match c {
            '/' => write!(writer, "\\/")?,
            '\\' => write!(writer, "\\\\")?,
            '"' => write!(writer, "\\\"")?,
            '\u{c}' => write!(writer, "\\f")?,
            '\u{8}' => write!(writer, "\\b")?,
            '\n' => write!(writer, "\\n")?,
            '\r' => write!(writer, "\\r")?,
            '\t' => write!(writer, "\\t")?,
            '\u{0}'..='\u{1f}' => {
                let hex = c as u16;
                write!(writer, "\\u00")?;
                for i in (0..2).rev() {
                    write!(writer, "{}", hex_char(hex, i))?;
                }
            }
            _ => write!(writer, "{c}")?,
        }
        Ok(())
    })
}

fn write_null<W: Write>(writer: &mut W) -> std::fmt::Result {
    write!(writer, "null")
}

fn write_boolean<W: Write>(writer: &mut W, value: bool) -> std::fmt::Result {
    write!(writer, "{value}")
}

fn write_number<W: Write>(writer: &mut W, value: f64) -> std::fmt::Result {
    write!(writer, "{value}")
}

fn write_string<W: Write>(writer: &mut W, value: &str) -> std::fmt::Result {
    write!(writer, "\"")?;
    write_escaped_string(writer, value)?;
    write!(writer, "\"")
}

fn write_array<W: Write>(writer: &mut W, array: &[Value], formatter: JsonFormatter) -> std::fmt::Result {
    write!(writer, "[")?;
    if !formatter.sameline {
        write!(writer, "\n")?;
    }
    let indent = formatter.indent();
    array.iter().enumerate().try_for_each(|(index, value)| {
        write!(writer, "{indent}")?;
        write_value(writer, value, indent)?;
        // Make sure it's not the final item.
        if index + 1 != array.len() {
            formatter.write_separator(writer)?;
        }
        Ok(())
    })?;
    if !formatter.sameline {
        write!(writer, "\n")?;
    }
    write!(writer, "{formatter}]")
}

fn write_object<W: Write>(writer: &mut W, object: &ValueMap, formatter: JsonFormatter) -> std::fmt::Result {
    write!(writer, "{{")?;
    if !formatter.sameline {
        write!(writer, "\n")?;
    }
    let indent = formatter.indent();
    object.iter().enumerate().try_for_each(|(index, (key, value))| {
        write!(writer, "{indent}")?;
        write_string(writer, key)?;
        if formatter.no_spacing {
            write!(writer, ":")?;
        } else {
            write!(writer, " : ")?;
        }
        write_value(writer, value, indent)?;
        // Make sure it's not the final item
        if index + 1 != object.len() {
            formatter.write_separator(writer)?;
        }
        Ok(())
    })?;
    if !formatter.sameline {
        write!(writer, "\n")?;
    }
    write!(writer, "{formatter}}}")
}

fn write_value<W: Write>(writer: &mut W, value: &Value, formatter: JsonFormatter) -> std::fmt::Result {
    match value {
        Value::Null => write_null(writer),
        &Value::Boolean(boolean) => write_boolean(writer, boolean),
        &Value::Number(number) => write_number(writer, number),
        Value::String(string) => write_string(writer, string),
        Value::Array(array) => write_array(writer, array, formatter),
        Value::Object(object) => write_object(writer, object, formatter),
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_value(f, self, JsonFormatter::default())
    }
}

impl Value {
    /// Converts [Value] to [String] with no whitespace.
    pub fn to_string_compressed(&self) -> String {
        self.to_string_formatted(JsonFormatter::new(true, true, Indent::None))
    }
    
    /// Converts [Value] to [String] with formatter.
    pub fn to_string_formatted(&self, formatter: JsonFormatter) -> String {
        let mut buffer = String::new();
        match write_value(&mut buffer, self, formatter) {
            Ok(()) => buffer,
            Err(_) => unreachable!()
        }
    }
}

#[test]
fn format_test() {
    let src = r#"{"number":-0.31415e1,"bool":[false,true,false,],"null":null,"string":"Hello, world!"}"#;
    let json = match Value::from_str(src) {
        Ok(json) => json,
        Err(error) => {
            println!("Error: {error}");
            return;
        }
    };
    let dump = json.to_string();
    println!("{dump}");
}