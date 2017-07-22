use packstream::values::Value;

pub trait CastValue {
    fn as_string(&self) -> Option<String>;
    fn as_i64(&self) -> Option<i64>;
    fn as_u64(&self) -> Option<u64>;
    fn as_i32(&self) -> Option<i32>;
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

    fn as_i64(&self) -> Option<i64> {
        match self {
            &Value::Integer(i) => Some(i),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            &Value::Integer(i) => Some(i as u64),
            _ => None,
        }
    }

    fn as_i32(&self) -> Option<i32> {
        match self {
            &Value::Integer(i) => Some(i as i32),
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
        match self {
            &Value::List(ref l) => Some(l),
            _ => None,
        }
    }
}
