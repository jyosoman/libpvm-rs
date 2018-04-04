use neo4j::Value;
use std::collections::HashMap;

use data::{Generable, HasID, HasUUID, NodeID};
use uuid::{IntoUUID, Uuid5};

#[derive(Clone, Debug)]
pub struct EditSession {
    db_id: NodeID,
    uuid: Uuid5,
    pub name: String,
}

pub struct EditInit {
    pub name: String,
}

impl EditSession {
    pub fn from_props(mut props: HashMap<String, Value>) -> Result<Self, &'static str> {
        Ok(EditSession {
            db_id: NodeID::new(props
                .remove("db_id")
                .and_then(Value::into_int)
                .ok_or("db_id property is missing or not an Integer")?),
            uuid: props
                .remove("uuid")
                .and_then(Value::into_uuid5)
                .ok_or("uuid property is missing or not a UUID5")?,
            name: props
                .remove("name")
                .and_then(Value::into_string)
                .ok_or("name property is missing or not a string")?,
        })
    }
}

impl HasID for EditSession {
    fn get_db_id(&self) -> NodeID {
        self.db_id
    }
}

impl Generable for EditSession {
    type Init = EditInit;

    fn new(id: NodeID, uuid: Uuid5, init: Option<Self::Init>) -> Self
    where
        Self: Sized,
    {
        match init {
            Some(i) => EditSession {
                db_id: id,
                uuid,
                name: i.name,
            },
            None => EditSession {
                db_id: id,
                uuid,
                name: String::new(),
            },
        }
    }
}

impl HasUUID for EditSession {
    fn get_uuid(&self) -> Uuid5 {
        self.uuid
    }
}
