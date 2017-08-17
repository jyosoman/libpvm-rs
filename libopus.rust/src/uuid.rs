use std::error::Error;
use std::fmt::{self, Display};
use std::num::ParseIntError;
use std::str::FromStr;
use std::string::ToString;

use packstream::values::{self, Value};
use serde::de::{self, Visitor, Deserialize, Deserializer};

#[derive(Debug, PartialEq, Hash, Clone)]
pub enum Uuid5 {
    Single(u128),
    Double([u128; 2]),
}

#[derive(Debug)]
pub enum Uuid5Error {
    Formatting(String),
    Parse(ParseIntError),
}

impl From<ParseIntError> for Uuid5Error {
    fn from(err: ParseIntError) -> Uuid5Error {
        Uuid5Error::Parse(err)
    }
}

impl Error for Uuid5Error {
    fn description(&self) -> &str {
        match *self {
            Uuid5Error::Formatting(ref s) => s,
            Uuid5Error::Parse(ref err) => err.description(),
        }
    }
}

impl Display for Uuid5Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Uuid5Error::Formatting(ref s) => write!(f, "{}", s),
            Uuid5Error::Parse(ref err) => write!(f, "{}", err),
        }
    }
}

impl Uuid5 {
    pub fn zero() -> Uuid5 {
        Uuid5::Single(0)
    }
}

impl FromStr for Uuid5 {
    type Err = Uuid5Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.len() {
            36 => {
                Ok(Uuid5::Single(
                    u128::from_str_radix(&s.replace("-", "")[..], 16)?,
                ))
            }
            73 => {
                let stripped = s.replace("-", "");
                Ok(Uuid5::Double([
                    u128::from_str_radix(&stripped[..32], 16)?,
                    u128::from_str_radix(&stripped[33..], 16)?,
                ]))
            }
            _ => Err(Uuid5Error::Formatting(
                format!("{} is an invalid UUID v5 format", s),
            )),
        }
    }
}

impl Display for Uuid5 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Uuid5::Single(v) => {
                let vf = format!("{:032x}", v);
                write!(
                    f,
                    "{}-{}-{}-{}-{}",
                    &vf[0..8],
                    &vf[8..12],
                    &vf[12..16],
                    &vf[16..20],
                    &vf[20..]
                )
            }
            Uuid5::Double([v, v1]) => {
                write!(
                    f,
                    "{}:{}",
                    Uuid5::Single(v).to_string(),
                    Uuid5::Single(v1).to_string()
                )
            }
        }
    }
}


impl values::ValueCast for Uuid5 {
    fn from(&self) -> Value {
        Value::String(self.to_string())
    }
}


struct Uuid5Visitor;

impl<'de> Visitor<'de> for Uuid5Visitor {
    type Value = Uuid5;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a Uuid5 (UUID version 5) value")
    }

    fn visit_string<E>(self, value: String) -> Result<Uuid5, E>
    where
        E: de::Error,
    {
        let pval_r = value.parse::<Uuid5>();
        match pval_r {
            Ok(pval) => Ok(pval),
            Err(e) => Err(E::custom(format!("Uuid5 parsing: {}", e.description()))),
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<Uuid5, E>
    where
        E: de::Error,
    {
        let pval_r = value.parse::<Uuid5>();
        match pval_r {
            Ok(pval) => Ok(pval),
            Err(e) => Err(E::custom(format!("Uuid5 parsing: {}", e.description()))),
        }
    }
}


impl<'de> Deserialize<'de> for Uuid5 {
    fn deserialize<D>(deserializer: D) -> Result<Uuid5, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(Uuid5Visitor)
    }
}
