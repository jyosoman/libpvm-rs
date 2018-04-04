use neo4j::Value;
use std::collections::HashMap;

use data::{Denumerate, Enumerable, Generable, HasID, HasUUID, NodeID, node_types::EnumNode};
use uuid::{IntoUUID, Uuid5};

#[derive(Clone, Debug)]
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

impl HasID for Process {
    fn get_db_id(&self) -> NodeID {
        self.db_id
    }
}

impl Enumerable for Process {
    fn enumerate(self) -> EnumNode {
        EnumNode::Proc(self)
    }
}

impl Denumerate for Process {
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
    type Init = ProcessInit;

    fn new(id: NodeID, uuid: Uuid5, init: Option<Self::Init>) -> Self
    where
        Self: Sized,
    {
        match init {
            Some(i) => Process {
                db_id: id,
                uuid,
                cmdline: i.cmdline,
                pid: i.pid,
                thin: i.thin,
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
