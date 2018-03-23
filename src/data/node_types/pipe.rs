use std::collections::HashMap;
use neo4j::Value;

use super::super::{Enumerable, Generable, HasID, HasUUID, NodeID, ToDB};
use super::EnumNode;
use uuid::{Uuid5, IntoUUID};

pub struct PipeInit {
    pub fd: i32
}

#[derive(Debug)]
pub struct Pipe {
    db_id: NodeID,
    uuid: Uuid5,
    pub fd: i32,
}

impl Pipe {
    pub fn from_props(mut props: HashMap<String, Value>) -> Result<Self, &'static str> {
        Ok(Pipe {
            db_id: NodeID::new(props
                .remove("db_id")
                .and_then(Value::into_int)
                .ok_or("db_id property is missing or not an Integer")?),
            uuid: props
                .remove("uuid")
                .and_then(Value::into_uuid5)
                .ok_or("uuid property is missing or not a UUID5")?,
            fd: props
                .remove("fd")
                .and_then(Value::into_int)
                .ok_or("fd property is missing or not an Integer")?,
        })
    }
}

impl ToDB for Pipe {
    fn to_db(&self) -> Value {
        hashmap!("db_id" => Value::from(self.db_id),
                 "uuid"  => Value::from(self.uuid),
                 "fd"    => Value::from(self.fd))
            .into()
    }
    fn get_labels(&self) -> Value {
        vec!["Node", "Pipe"].into()
    }
}

impl HasID for Pipe {
    fn get_db_id(&self) -> NodeID {
        self.db_id
    }
}

impl Enumerable for Pipe {
    fn enumerate(self) -> EnumNode {
        EnumNode::Pipe(self)
    }
    fn denumerate(val: &EnumNode) -> &Self {
        if let EnumNode::Pipe(ref p) = *val {
            p
        } else {
            panic!("{:?} is not a pipe", val)
        }
    }
    fn denumerate_mut(val: &mut EnumNode) -> &mut Self {
        if let EnumNode::Pipe(ref mut p) = *val {
            p
        } else {
            panic!("{:?} is not a pipe", val)
        }
    }
}

impl Generable for Pipe {
    type Additional = PipeInit;

    fn new(id: NodeID, uuid: Uuid5, additional: Option<Self::Additional>) -> Self
    where
        Self: Sized,
    {
        match additional {
            Some(add) => Pipe {
                db_id: id,
                uuid,
                fd: add.fd,
            },
            None => Pipe {
                db_id: id,
                uuid,
                fd: -1,
            },
        }
    }
}

impl HasUUID for Pipe {
    fn get_uuid(&self) -> Uuid5 {
        self.uuid
    }
}
