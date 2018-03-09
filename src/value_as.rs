use std::convert::{TryFrom, TryInto};
use std::collections::HashMap;
use std::str::FromStr;

use packstream::values::Value;

use uuid::Uuid5;

pub trait CastValue {
    fn into_string(self) -> Option<String>;
    fn into_int<T: TryFrom<i64>>(self) -> Option<T>;
    fn into_float<T: TryFrom<f64>>(self) -> Option<T>;
    fn into_bool(self) -> Option<bool>;
    fn into_vec(self) -> Option<Vec<Value>>;
    fn into_map(self) -> Option<HashMap<String, Value>>;
    fn into_uuid5(self) -> Option<Uuid5>;
}


impl CastValue for Value {
    fn into_bool(self) -> Option<bool> {
        match self {
            Value::Boolean(v) => Some(v),
            _ => None,
        }
    }

    fn into_int<T: TryFrom<i64>>(self) -> Option<T> {
        match self {
            Value::Integer(v) => v.try_into().ok(),
            _ => None,
        }
    }

    fn into_float<T: TryFrom<f64>>(self) -> Option<T> {
        match self {
            Value::Float(v) => v.try_into().ok(),
            _ => None,
        }
    }

    fn into_string(self) -> Option<String> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }

    fn into_vec(self) -> Option<Vec<Value>> {
        match self {
            Value::List(v) => Some(v),
            _ => None,
        }
    }

    fn into_map(self) -> Option<HashMap<String, Value>> {
        match self {
            Value::Map(v) => Some(v),
            _ => None,
        }
    }

    fn into_uuid5(self) -> Option<Uuid5> {
        match self {
            Value::String(s) => Uuid5::from_str(&s).ok(),
            _ => None,
        }
    }
}
