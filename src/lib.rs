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

pub trait IndexOrKey {
    fn get(self, value: &Value) -> Option<&Value>;
    fn get_mut(self, value: &mut Value) -> Option<&mut Value>;
    fn get_or_insert(self, value: &mut Value) -> &mut Value;
}

impl IndexOrKey for usize {
    fn get(self, value: &Value) -> Option<&Value> {
        let Value::Array(array) = value else {
            return None;
        };
        array.get(self)
    }

    fn get_mut(self, value: &mut Value) -> Option<&mut Value> {
        let Value::Array(array) = value else {
            return None;
        };
        array.get_mut(self)
    }

    fn get_or_insert(self, _value: &mut Value) -> &mut Value {
        panic!();
    }
}

impl IndexOrKey for &str {
    fn get(self, value: &Value) -> Option<&Value> {
        let Value::Object(object) = value else {
            return None;
        };
        object.get(self)
    }

    fn get_mut(self, value: &mut Value) -> Option<&mut Value> {
        let Value::Object(object) = value else {
            return None;
        };
        object.get_mut(self)
    }

    fn get_or_insert(self, value: &mut Value) -> &mut Value {
        if let Value::Null = value {
            *value = Value::Object(ValueMap::new());
        }
        match value {
            Value::Object(object) => object.entry(self.to_owned()).or_insert(Value::Null),
            _ => panic!()
        }
    }
}

impl IndexOrKey for String {
    fn get(self, value: &Value) -> Option<&Value> {
        let Value::Object(object) = value else {
            return None;
        };
        object.get(&self)
    }

    fn get_mut(self, value: &mut Value) -> Option<&mut Value> {
        let Value::Object(object) = value else {
            return None;
        };
        object.get_mut(&self)
    }

    fn get_or_insert(self, value: &mut Value) -> &mut Value {
        if let Value::Null = value {
            *value = Value::Object(ValueMap::new());
        }
        match value {
            Value::Object(object) => object.entry(self).or_insert(Value::Null),
            _ => panic!()
        }
    }
}

impl Value {
    pub fn push<T: Into<Value>>(&mut self, value: T) {
        let Value::Array(array) = self else {    
            panic!("Not an array.");
        };
        array.push(value.into());
    }

    pub fn insert<T: Into<Value>>(&mut self, k: String, v: T) -> Option<Value> {
        let Value::Object(object) = self else {
            panic!("Not an object.");
        };
        object.insert(k, v.into())
    }

    pub fn get<I: IndexOrKey>(&self, i_k: I) -> Option<&Value> {
        i_k.get(self)
    }

    pub fn get_mut<I: IndexOrKey>(&mut self, i_k: I) -> Option<&mut Value> {
        i_k.get_mut(self)
    }
}

impl<I: IndexOrKey> std::ops::Index<I> for Value {
    type Output = Value;
    fn index(&self, index: I) -> &Self::Output {
        static NULL: Value = Value::Null;
        index.get(self).unwrap_or(&NULL)
    }
}

impl<I: IndexOrKey> std::ops::IndexMut<I> for Value {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.get_or_insert(self)
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