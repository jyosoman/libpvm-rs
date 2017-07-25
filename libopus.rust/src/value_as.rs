use std::convert::{TryFrom, TryInto};

use packstream::values::Value;

pub trait CastValue {
    fn as_string(&self) -> Option<String>;
    fn as_int<T: TryFrom<i64>>(&self) -> Option<T>;
    fn as_bool(&self) -> Option<bool>;
    fn as_vec_ref(&self) -> Option<&Vec<Value>>;
}

impl CastValue for Value {
    fn as_string(&self) -> Option<String> {
        match self {
            &Value::String(ref s) => Some(s.clone()),
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
        match self {
            &Value::Boolean(b) => Some(b),
            _ => None,
        }
    }

    fn as_vec_ref(&self) -> Option<&Vec<Value>> {
        match *self {
            Value::List(ref l) => Some(l),
            _ => None,
        }
    }
}
