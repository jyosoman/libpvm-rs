use neo4j::Value;
use std::collections::HashMap;

use data::{Generable, HasID, HasUUID, NodeID};
use uuid::{IntoUUID, Uuid5};

pub struct PipeInit {
    pub fd: i32,
}

#[derive(Clone, Debug)]
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

impl HasID for Pipe {
    fn get_db_id(&self) -> NodeID {
        self.db_id
    }
}

impl Generable for Pipe {
    type Init = PipeInit;

    fn new(id: NodeID, uuid: Uuid5, init: Option<Self::Init>) -> Self
    where
        Self: Sized,
    {
        match init {
            Some(i) => Pipe {
                db_id: id,
                uuid,
                fd: i.fd,
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
