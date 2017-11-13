use std::convert::{TryFrom, TryInto};
use std::collections::HashMap;
use std::str::FromStr;

use packstream::values::Value;

use uuid::Uuid5;

pub trait CastValue {
    fn as_string(&self) -> Option<String>;
    fn as_int<T: TryFrom<i64>>(&self) -> Option<T>;
    fn as_bool(&self) -> Option<bool>;
    fn as_vec(self) -> Option<Vec<Value>>;
    fn as_map(self) -> Option<HashMap<String, Value>>;
    fn as_uuid5(&self) -> Option<Uuid5>;
}

impl CastValue for Value {
    fn as_string(&self) -> Option<String> {
        match *self {
            Value::String(ref s) => Some(s.clone()),
            _ => None,
        }
    }

    fn as_int<T: TryFrom<i64>>(&self) -> Option<T> {
        match *self {
            Value::Integer(i) => i.try_into().ok(),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Boolean(b) => Some(b),
            _ => None,
        }
    }

    fn as_vec(self) -> Option<Vec<Value>> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    fn as_map(self) -> Option<HashMap<String, Value>> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    fn as_uuid5(&self) -> Option<Uuid5> {
        match *self {
            Value::String(ref s) => Uuid5::from_str(s).ok(),
            _ => None,
        }
    }
}
