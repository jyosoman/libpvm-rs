use std::collections::HashMap;
use packstream::values::Value;
use value_as::CastValue;

use super::super::{Enumerable, Generable, HasID, HasUUID, NodeID, ToDB};
use super::EnumNode;
use uuid::Uuid5;

#[derive(Debug)]
pub struct Process {
    pub db_id: NodeID,
    pub uuid: Uuid5,
    pub pid: i32,
    pub cmdline: String,
    pub thin: bool,
}

impl Process {
    pub fn from_props(props: &HashMap<String, Value>) -> Result<Self, &'static str> {
        Ok(Process {
            db_id: NodeID::new(props
                .get("db_id")
                .and_then(Value::as_int)
                .ok_or("db_id property is missing or not an Integer")?),
            uuid: props
                .get("uuid")
                .and_then(Value::as_uuid5)
                .ok_or("uuid property is missing or not a UUID5")?,
            cmdline: props
                .get("cmdline")
                .and_then(Value::as_string)
                .ok_or("cmdline property is missing or not a String")?,
            pid: props
                .get("pid")
                .and_then(Value::as_int)
                .ok_or("pid property is missing or not an Integer")?,
            thin: props
                .get("thin")
                .and_then(Value::as_bool)
                .ok_or("thin property is missing or not a bool")?,
        })
    }
}

impl ToDB for Process {
    fn to_db(&self) -> Value {
        hashmap!("db_id"   => Value::from(self.db_id),
                 "uuid"    => Value::from(self.uuid),
                 "cmdline" => Value::from(self.cmdline.clone()),
                 "pid"     => Value::from(self.pid),
                 "thin"    => Value::from(self.thin))
            .into()
    }
    fn get_labels(&self) -> Value {
        vec!["Node", "Process"].into()
    }
}

impl HasID for Process {
    fn get_db_id(&self) -> NodeID {
        self.db_id
    }
}

impl Enumerable for Process {
    fn enumerate(self) -> EnumNode {
        EnumNode::Proc(self)
    }
    fn denumerate(val: EnumNode) -> Self
    where
        Self: Sized,
    {
        if let EnumNode::Proc(pro) = val {
            pro
        } else {
            panic!()
        }
    }
}

impl Generable for Process {
    fn new(id: NodeID, uuid: Uuid5, additional: Option<HashMap<&'static str, Value>>) -> Self
    where
        Self: Sized,
    {
        let mut p = Process {
            db_id: id,
            uuid,
            cmdline: String::new(),
            pid: 0,
            thin: true,
        };
        if let Some(map) = additional {
            if let Some(v) = map.get("pid") {
                p.pid = v.as_int().unwrap();
            }
            if let Some(v) = map.get("cmdline") {
                p.cmdline = v.as_string().unwrap();
            }
            if let Some(v) = map.get("thin") {
                p.thin = v.as_bool().unwrap();
            }
        }
        p
    }
}

impl HasUUID for Process {
    fn get_uuid(&self) -> Uuid5 {
        self.uuid
    }
}
