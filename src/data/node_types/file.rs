use std::collections::HashMap;
use packstream::values::Value;
use value_as::CastValue;

use super::super::{Enumerable, Generable, HasID, HasUUID, NodeID, ToDB};
use super::EnumNode;
use uuid::Uuid5;

pub struct FileInit {
    pub name: String,
}

#[derive(Debug)]
pub struct File {
    pub db_id: NodeID,
    pub uuid: Uuid5,
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
    fn to_db(&self) -> Value {
        hashmap!("db_id" => Value::from(self.db_id),
                 "uuid"  => Value::from(self.uuid),
                 "name"  => Value::from(self.name.clone()))
            .into()
    }
    fn get_labels(&self) -> Value {
        vec!["Node", "File"].into()
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
    fn denumerate(val: EnumNode) -> Self
    where
        Self: Sized,
    {
        if let EnumNode::File(f) = val {
            f
        } else {
            panic!()
        }
    }
}

impl Generable for File {
    type Additional = FileInit;

    fn new(id: NodeID, uuid: Uuid5, additional: Option<Self::Additional>) -> Self
    where
        Self: Sized,
    {
        let mut f = File {
            db_id: id,
            uuid,
            name: String::new(),
        };
        if let Some(add) = additional {
            f.name = add.name;
        }
        f
    }
}

impl HasUUID for File {
    fn get_uuid(&self) -> Uuid5 {
        self.uuid
    }
}
