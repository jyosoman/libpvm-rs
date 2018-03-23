use std::collections::HashMap;
use neo4j::Value;

use super::super::{Enumerable, Generable, HasID, HasUUID, NodeID, ToDB};
use super::EnumNode;
use uuid::{Uuid5, IntoUUID};

#[derive(Debug)]
pub struct Process {
    db_id: NodeID,
    uuid: Uuid5,
    pub pid: i32,
    pub cmdline: String,
    pub thin: bool,
}

pub struct ProcessInit {
    pub pid: i32,
    pub cmdline: String,
    pub thin: bool,
}

impl Process {
    pub fn from_props(mut props: HashMap<String, Value>) -> Result<Self, &'static str> {
        Ok(Process {
            db_id: NodeID::new(props
                .remove("db_id")
                .and_then(Value::into_int)
                .ok_or("db_id property is missing or not an Integer")?),
            uuid: props
                .remove("uuid")
                .and_then(Value::into_uuid5)
                .ok_or("uuid property is missing or not a UUID5")?,
            cmdline: props
                .remove("cmdline")
                .and_then(Value::into_string)
                .ok_or("cmdline property is missing or not a String")?,
            pid: props
                .remove("pid")
                .and_then(Value::into_int)
                .ok_or("pid property is missing or not an Integer")?,
            thin: props
                .remove("thin")
                .and_then(Value::into_bool)
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
    fn denumerate(val: &EnumNode) -> &Self {
        if let EnumNode::Proc(ref pro) = *val {
            pro
        } else {
            panic!("{:?} is not a process", val)
        }
    }
    fn denumerate_mut(val: &mut EnumNode) -> &mut Self {
        if let EnumNode::Proc(ref mut pro) = *val {
            pro
        } else {
            panic!("{:?} is not a process", val)
        }
    }
}

impl Generable for Process {
    type Additional = ProcessInit;

    fn new(id: NodeID, uuid: Uuid5, additional: Option<Self::Additional>) -> Self
    where
        Self: Sized,
    {
        match additional {
            Some(add) => Process {
                db_id: id,
                uuid,
                cmdline: add.cmdline,
                pid: add.pid,
                thin: add.thin,
            },
            None => Process {
                db_id: id,
                uuid,
                cmdline: String::new(),
                pid: 0,
                thin: true,
            },
        }
    }
}

impl HasUUID for Process {
    fn get_uuid(&self) -> Uuid5 {
        self.uuid
    }
}
