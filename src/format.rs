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
    Spaces(u8),
    Tabs(u8),
}

impl std::fmt::Display for Indent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const SPACES: &'static str = "                                                                                                                                                                                                                                                                ";
        const TABS: &'static str = "\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t";
        match self {
            &Self::Spaces(count) => write!(f, "{}", &SPACES[..count as usize])?,
            &Self::Tabs(count) => write!(f, "{}", &TABS[..count as usize])?,
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct JsonFormatter {
    /// All on the same line.
    sameline: bool,
    /// No spaces between elements or around colons.
    no_spacing: bool,
    /// The [Indent] to use. This is ignored if `sameline` is true.
    indent: Indent,
    /// Indent level. Only modify this if you know what you're doing.
    indent_level: u32,
}

struct Indentation<'a>(&'a JsonFormatter);

impl<'a> std::fmt::Display for Indentation<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.0.indent_level {
            write!(f, "{}", self.0.indent)?;
        }
        Ok(())
    }
}

impl JsonFormatter {
    fn new(sameline: bool, no_spacing: bool, indent: Indent) -> Self {
        Self::new_indented(0, sameline, no_spacing, indent)
    }

    fn new_indented(indent_level: u32, sameline: bool, no_spacing: bool, indent: Indent) -> Self {
        Self {
            sameline,
            no_spacing,
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
    // Writing to a String is infallible (I think), so this should never fail.
    write_escaped_string(&mut buffer, s).unwrap();
    buffer
}

fn write_escaped_string<W: Write, S: AsRef<str>>(writer: &mut W, s: S) -> std::fmt::Result {
    s.as_ref().chars().try_for_each(|c| {
        match c {
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
        if !indent.sameline {
            write!(writer, "{}", indent.indentation())?;
        }
        write_value(writer, value, indent)?;
        // Make sure it's not the final item.
        if index + 1 != array.len() {
            indent.write_separator(writer)?;
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
        if indent.no_spacing {
            write!(writer, ":")?;
        } else {
            write!(writer, " : ")?;
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_value(f, self, JsonFormatter::new(true, true, Indent::Spaces(0)))
    }
}

pub struct PrettyPrint<'a>(&'a Value, Indent, bool);

impl<'a> std::fmt::Display for PrettyPrint<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_value(f, self.0, JsonFormatter::new(false, self.2, self.1))
    }
}

impl Value {

    /// Returns an object suitable for pretty printing.
    /// #### Arguments:
    /// - `indent`: Controls the indentation. Use `Indent::Spaces(0)` if you don't want indentation (This defeats the purpose of pretty printing).
    /// - `spacing`: Determines whether or not there are spaces before and after colons.
    pub fn pretty_print_format(&self, indent: Indent, spacing: bool) -> PrettyPrint<'_> {
        PrettyPrint(self, indent, !spacing)
    }

    pub fn pretty_print(&self) -> PrettyPrint<'_> {
        PrettyPrint(self, Indent::Spaces(2), false)
    }
}