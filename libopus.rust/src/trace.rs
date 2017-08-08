use std::fmt::{self, Display};
use std::error::Error;
use std::num::ParseIntError;
use std::str::FromStr;
use packstream::values::{ValueCast, Value};
use serde::de::{self, Visitor, Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
pub struct TraceEventOld {
    pub event: String,
    pub host: Option<String>,
    pub time: u64,
    pub pid: i32,
    pub ppid: i32,
    pub tid: i32,
    pub uid: i32,
    pub exec: Option<String>,
    pub cmdline: Option<String>,
    pub upath1: Option<String>,
    pub upath2: Option<String>,
    pub fd: Option<i32>,
    pub flags: Option<i32>,
    pub fdpath: Option<String>,
    pub subjprocuuid: String,
    pub subjthruuid: String,
    pub arg_objuuid1: Option<String>,
    pub arg_objuuid2: Option<String>,
    pub ret_objuuid1: Option<String>,
    pub ret_objuuid2: Option<String>,
    pub retval: i32,
}

#[derive(Debug, PartialEq, Hash, Clone)]
pub struct uuid5(u128);

impl uuid5 {
    pub fn zero() -> uuid5 {
        uuid5(0)
    }
}

impl FromStr for uuid5 {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut uuid_numstr = String::with_capacity(32);
        let uuid_format_sz = [8, 4, 4, 4, 12, 0];
        let mut start = 0;
        let mut end = uuid_format_sz[0];
        for i in 1..uuid_format_sz.len() {
            uuid_numstr.push_str(&s[start..end]);
            start += uuid_format_sz[i-1] + 1; // skip "-"
            end += uuid_format_sz[i] + 1;
        }
        Ok(uuid5(u128::from_str_radix(&uuid_numstr[..], 16)?))
    }
}

impl Display for uuid5 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val = self.0;
        let vf = format!("{:x}", val);
        write!(f, "{}-{}-{}-{}-{}", &vf[0..8], &vf[8..12], &vf[12..16], &vf[16..20], &vf[20..])
    }
}


impl ValueCast for uuid5 {
    fn from(&self) -> Value {
        Value::String(format!("{}", self))
    }
}


struct UUID5Visitor;

impl<'de> Visitor<'de> for UUID5Visitor {
    type Value = uuid5;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a uuid5 (UUID version 5) value")
    }

    fn visit_string<E>(self, value: String) -> Result<uuid5, E>
        where E: de::Error
    {
        let pval_r = uuid5::from_str(&value[..]);
        match pval_r {
            Ok(pval) => Ok(pval),
            Err(e) => Err(E::custom(format!("uuid5 parsing: {}", e.description()))),
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<uuid5, E>
        where E: de::Error
    {
        let pval_r = uuid5::from_str(value);
        match pval_r {
            Ok(pval) => Ok(pval),
            Err(e) => Err(E::custom(format!("uuid5 parsing: {}", e.description()))),
        }
    }

}


impl<'de> Deserialize<'de> for uuid5 {
    fn deserialize<D>(deserializer: D) -> Result<uuid5, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_string(UUID5Visitor)
    }
}

#[derive(Deserialize, Debug)]
pub struct TraceEvent {
    pub event: String,
    pub host: Option<String>,
    pub time: u64,
    pub pid: i32,
    pub ppid: i32,
    pub tid: i32,
    pub uid: i32,
    pub exec: Option<String>,
    pub cmdline: Option<String>,
    pub upath1: Option<String>,
    pub upath2: Option<String>,
    pub fd: Option<i32>,
    pub flags: Option<i32>,
    pub fdpath: Option<String>,
    pub subjprocuuid: uuid5,
    pub subjthruuid: uuid5,
    pub arg_objuuid1: Option<uuid5>,
    pub arg_objuuid2: Option<uuid5>,
    pub ret_objuuid1: Option<uuid5>,
    pub ret_objuuid2: Option<uuid5>,
    pub retval: i32,
}
