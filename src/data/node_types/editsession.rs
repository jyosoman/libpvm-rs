use neo4j::Value;
use std::collections::HashMap;

use data::{Denumerate, Enumerable, Generable, HasID, HasUUID, NodeID, ToDB, node_types::EnumNode};
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

impl ToDB for EditSession {
    fn get_labels(&self) -> Vec<&'static str> {
        vec!["Node", "EditSession"]
    }
    fn get_props(&self) -> HashMap<&'static str, Value> {
        hashmap!("uuid"  => Value::from(self.uuid),
                 "name"  => Value::from(self.name.clone()))
    }
}

impl HasID for EditSession {
    fn get_db_id(&self) -> NodeID {
        self.db_id
    }
}

impl Enumerable for EditSession {
    fn enumerate(self) -> EnumNode {
        EnumNode::EditSession(self)
    }
}

impl Denumerate for EditSession {
    fn denumerate(val: &EnumNode) -> &Self {
        if let EnumNode::EditSession(ref ed) = *val {
            ed
        } else {
            panic!("{:?} is not an editsession", val)
        }
    }
    fn denumerate_mut(val: &mut EnumNode) -> &mut Self {
        if let EnumNode::EditSession(ref mut ed) = *val {
            ed
        } else {
            panic!("{:?} is not an editsession", val)
        }
    }
}

impl Generable for EditSession {
    type Additional = EditInit;

    fn new(id: NodeID, uuid: Uuid5, additional: Option<Self::Additional>) -> Self
    where
        Self: Sized,
    {
        match additional {
            Some(add) => EditSession {
                db_id: id,
                uuid,
                name: add.name,
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
