use neo4j::Value;
use std::collections::HashMap;

use data::{Denumerate, Enumerable, Generable, HasID, HasUUID, NodeID, node_types::EnumNode};
use uuid::{IntoUUID, Uuid5};

pub struct FileInit {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct File {
    db_id: NodeID,
    uuid: Uuid5,
    pub name: String,
}

impl File {
    pub fn from_props(mut props: HashMap<String, Value>) -> Result<Self, &'static str> {
        Ok(File {
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

impl HasID for File {
    fn get_db_id(&self) -> NodeID {
        self.db_id
    }
}

impl Enumerable for File {
    fn enumerate(self) -> EnumNode {
        EnumNode::File(self)
    }
}

impl Denumerate for File {
    fn denumerate(val: &EnumNode) -> &Self {
        if let EnumNode::File(ref f) = *val {
            f
        } else {
            panic!("{:?} is not a file", val)
        }
    }
    fn denumerate_mut(val: &mut EnumNode) -> &mut Self {
        if let EnumNode::File(ref mut f) = *val {
            f
        } else {
            panic!("{:?} is not a file", val)
        }
    }
}

impl Generable for File {
    type Init = FileInit;

    fn new(id: NodeID, uuid: Uuid5, init: Option<Self::Init>) -> Self
    where
        Self: Sized,
    {
        match init {
            Some(i) => File {
                db_id: id,
                uuid,
                name: i.name,
            },
            None => File {
                db_id: id,
                uuid,
                name: String::new(),
            },
        }
    }
}

impl HasUUID for File {
    fn get_uuid(&self) -> Uuid5 {
        self.uuid
    }
}
