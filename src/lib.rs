pub mod error;
pub mod parse;
pub mod format;
pub use bournemacro::json;

#[cfg(not(feature = "preserve_order"))]
pub type ValueMap = std::collections::HashMap<String, Value>;
#[cfg(feature = "preserve_order")]
pub type ValueMap = indexmap::IndexMap<String, Value>;

/// JSON Value.
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(ValueMap),
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_owned())
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::Array(value)
    }
}

impl From<ValueMap> for Value {
    fn from(value: ValueMap) -> Self {
        Value::Object(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Number(value as f64)
    }
}

pub trait ArrayExt {
    fn push_value<T: Into<Value>>(&mut self, value: T);
}

impl ArrayExt for Vec<Value> {
    fn push_value<T: Into<Value>>(&mut self, value: T) {
        self.push(value.into());
    }
}

pub trait ObjectExt {
    fn insert_value<T: Into<Value>>(&mut self, key: String, value: T) -> Option<Value>;
}

impl ObjectExt for ValueMap {
    fn insert_value<T: Into<Value>>(&mut self, k: String, v: T) -> Option<Value> {
        self.insert(k, v.into())
    }
}