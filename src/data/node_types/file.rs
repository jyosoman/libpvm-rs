use neo4j::Value;
use std::collections::HashMap;

use super::super::{Enumerable, Generable, HasID, HasUUID, NodeID, ToDB};
use super::EnumNode;
use uuid::{IntoUUID, Uuid5};

pub struct FileInit {
    pub name: String,
}

#[derive(Debug)]
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

impl ToDB for File {
    fn get_labels(&self) -> Vec<&'static str> {
        vec!["Node", "File"]
    }
    fn get_props(&self) -> HashMap<&'static str, Value> {
        hashmap!("uuid"  => Value::from(self.uuid),
                 "name"  => Value::from(self.name.clone()))
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
    type Additional = FileInit;

    fn new(id: NodeID, uuid: Uuid5, additional: Option<Self::Additional>) -> Self
    where
        Self: Sized,
    {
        match additional {
            Some(add) => File {
                db_id: id,
                uuid,
                name: add.name,
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
