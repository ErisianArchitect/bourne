#![allow(unused)]

use core::str;
use std::fmt::{
    Write,
    Formatter,
};
use std::str::FromStr;

use crate::error::*;
use crate::{
    Value,
    ValueMap,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Indent {
    Spaces(u8),
    Tabs(u8),
}

impl std::fmt::Display for Indent {
    /// Writes an [Indent] to a [Formatter]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // SAFETY: Creation of valid utf-8 string from byte array of spaces/tabs.
        const SPACES: &'static str = unsafe { str::from_utf8_unchecked(&[b' '; 256]) };
        const TABS: &'static str = unsafe { str::from_utf8_unchecked(&[b'\t'; 256]) };
        match self {
            &Self::Spaces(count) => write!(f, "{}", &SPACES[..count as usize]),
            &Self::Tabs(count) => write!(f, "{}", &TABS[..count as usize]),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct JsonFormatter {
    /// All on the same line.
    sameline: bool,
    /// No spaces between elements or around colons.
    spacing: bool,
    /// The [Indent] to use. This is ignored if `sameline` is true.
    indent: Indent,
    /// Indent level. Only modify this if you know what you're doing.
    indent_level: u32,
}

struct Indentation<'a>(&'a JsonFormatter);

impl<'a> std::fmt::Display for Indentation<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.0.indent_level {
            write!(f, "{}", self.0.indent)?;
        }
        Ok(())
    }
}

impl JsonFormatter {
    fn new(sameline: bool, spacing: bool, indent: Indent) -> Self {
        Self::new_indented(0, sameline, spacing, indent)
    }

    fn new_indented(indent_level: u32, sameline: bool, spacing: bool, indent: Indent) -> Self {
        Self {
            sameline,
            spacing,
            indent,
            indent_level: indent_level,
        }
    }

    /// Creates a copy of self where the indent level is incremented by 1.
    fn indent(&self) -> Self {
        let mut indent = self.clone();
        indent.indent_level += 1;
        indent
    }

    #[inline(always)]
    fn indentation(&self) -> Indentation<'_> {
        Indentation(self)
    }

    /// Writes the indentation to a writer.
    fn write_indent<W: Write>(&self, writer: &mut W) -> std::fmt::Result {
        write!(writer, "{}", self.indentation())
    }

    fn write_separator<W: Write>(&self, writer: &mut W) -> std::fmt::Result {
        write!(writer, ",")?;
        if !self.sameline {
            write!(writer, "\n")?;
        // Are double-negatives allowed in programming? There's not no spacing here.
        } else if self.spacing {
            write!(writer, " ")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for JsonFormatter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

fn hex_char(value: u16, slot: usize, lower_case: bool) -> char {
    const HEX_CHARS_UPPER: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'];
    const HEX_CHARS_LOWER: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
    let shift = slot * 4;
    let index = ((value & (0xf << shift)) >> shift) as usize;
    if lower_case {
        HEX_CHARS_LOWER[index]
    } else {
        HEX_CHARS_UPPER[index]
    }
}

/// Measures the length of a string after being escaped.
pub fn measure_escaped_string<S: AsRef<str>>(s: S) -> usize {
    s.as_ref().chars().map(|c| {
        match c {
            '\\' => 2,
            '"' => 2,
            '\u{000c}' => 2,
            '\u{0008}' => 2,
            '\n' => 2,
            '\r' => 2,
            '\t' => 2,
            '\u{0000}'..='\u{001f}' => 6,
            _ => c.len_utf8(),
        }
    }).sum()
}

/// Escapes a string.
pub fn escape_string<S: AsRef<str>>(s: S) -> String {
    let mut buffer = String::with_capacity(measure_escaped_string(s.as_ref()));
    // Writing to a String is infallible (I think), so this should never fail.
    write_escaped_string(&mut buffer, s).unwrap();
    buffer
}

fn write_escaped_string<W: Write, S: AsRef<str>>(writer: &mut W, s: S) -> std::fmt::Result {
    s.as_ref().chars().try_for_each(|c| {
        match c {
            '\\' => write!(writer, "\\\\")?,
            '"' => write!(writer, "\\\"")?,
            '\u{000c}' => write!(writer, "\\f")?,
            '\u{0008}' => write!(writer, "\\b")?,
            '\n' => write!(writer, "\\n")?,
            '\r' => write!(writer, "\\r")?,
            '\t' => write!(writer, "\\t")?,
            '\u{0000}'..='\u{001f}' => {
                let hex = c as u16;
                write!(writer, "\\u")?;
                for i in (0..4).rev() {
                    write!(writer, "{}", hex_char(hex, i, true))?;
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
    let indented_formatter = formatter.indent();
    array.iter().enumerate().try_for_each(|(index, value)| {
        if !indented_formatter.sameline {
            write!(writer, "{}", indented_formatter.indentation())?;
        }
        write_value(writer, value, indented_formatter)?;
        // Make sure it's not the final item.
        if index + 1 != array.len() {
            indented_formatter.write_separator(writer)?;
        }
        Ok(())
    })?;
    if !formatter.sameline {
        write!(writer, "\n")?;
        write!(writer, "{}", formatter.indentation())?;
    }
    
    write!(writer, "]")
}

fn write_object<W: Write>(writer: &mut W, object: &ValueMap, formatter: JsonFormatter) -> std::fmt::Result {
    write!(writer, "{{")?;
    if !formatter.sameline {
        write!(writer, "\n")?;
    }
    let indent = formatter.indent();
    object.iter().enumerate().try_for_each(|(index, (key, value))| {
        if !indent.sameline {
            write!(writer, "{}", indent.indentation())?;
        }
        write_string(writer, key)?;
        if indent.spacing {
            write!(writer, " : ")?;
        } else {
            write!(writer, ":")?;
        }
        write_value(writer, value, indent)?;
        // Make sure it's not the final item
        if index + 1 != object.len() {
            indent.write_separator(writer)?;
        }
        Ok(())
    })?;
    if !formatter.sameline {
        write!(writer, "\n")?;
        write!(writer, "{}", formatter.indentation())?;
    }
    write!(writer, "}}")
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_value(f, self, JsonFormatter::new(true, false, Indent::Spaces(0)))
    }
}

pub struct PrettyPrint<'a>(&'a Value, Indent, bool);

impl<'a> std::fmt::Display for PrettyPrint<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_value(f, self.0, JsonFormatter::new(false, self.2, self.1))
    }
}

impl Value {

    /// Returns an object suitable for pretty printing.
    /// #### Arguments:
    /// - `indent`: Controls the indentation. Use `Indent::Spaces(0)` if you don't want indentation (This defeats the purpose of pretty printing).
    /// - `spacing`: Determines whether or not there are spaces before and after colons.
    pub fn pretty_print_format(&self, indent: Indent, spacing: bool) -> PrettyPrint<'_> {
        PrettyPrint(self, indent, spacing)
    }

    /// Returns the default pretty printer.
    pub fn pretty_print(&self) -> PrettyPrint<'_> {
        PrettyPrint(self, Indent::Spaces(4), true)
    }
}