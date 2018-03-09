use std::collections::HashMap;
use packstream::values::Value;
use value_as::CastValue;

use super::super::{Enumerable, Generable, HasID, HasUUID, NodeID, ToDB};
use super::EnumNode;
use uuid::Uuid5;

#[derive(Clone, Copy, Debug)]
pub enum SocketClass {
    Unknown = 0,
    AfInet = 1,
    AfUnix = 2,
}

#[derive(Debug)]
pub struct Socket {
    pub db_id: NodeID,
    pub uuid: Uuid5,
    pub class: SocketClass,
    pub path: String,
    pub ip: String,
    pub port: u16,
}

pub struct SocketInit {
    pub class: SocketClass,
    pub path: String,
    pub ip: String,
    pub port: u16,
}

fn int_to_sock_class(val: i64) -> Option<SocketClass> {
    match val {
        0 => Some(SocketClass::Unknown),
        1 => Some(SocketClass::AfInet),
        2 => Some(SocketClass::AfUnix),
        _ => None,
    }
}

impl Socket {
    pub fn from_props(mut props: HashMap<String, Value>) -> Result<Self, &'static str> {
        Ok(Socket {
            db_id: NodeID::new(props
                .remove("db_id")
                .and_then(Value::into_int)
                .ok_or("db_id property is missing or not an Integer")?),
            uuid: props
                .remove("uuid")
                .and_then(Value::into_uuid5)
                .ok_or("uuid property is missing or not a UUID5")?,
            class: props
                .remove("class")
                .and_then(Value::into_int)
                .and_then(int_to_sock_class)
                .ok_or("class property is missing or not an Integer")?,
            path: props
                .remove("path")
                .and_then(Value::into_string)
                .ok_or("path property is missing or not a string")?,
            ip: props
                .remove("ip")
                .and_then(Value::into_string)
                .ok_or("ip property is missing or not a string")?,
            port: props
                .remove("port")
                .and_then(Value::into_int)
                .ok_or("port property is missing or not an Integer")?,
        })
    }
}

impl ToDB for Socket {
    fn to_db(&self) -> Value {
        hashmap!("db_id" => Value::from(self.db_id),
                 "uuid"  => Value::from(self.uuid),
                 "class"  => Value::from(self.class as i64),
                 "path" => Value::from(self.path.clone()),
                 "ip" => Value::from(self.ip.clone()),
                 "port" => Value::from(self.port))
            .into()
    }
    fn get_labels(&self) -> Value {
        vec!["Node", "Socket"].into()
    }
}

impl HasID for Socket {
    fn get_db_id(&self) -> NodeID {
        self.db_id
    }
}

impl Enumerable for Socket {
    fn enumerate(self) -> EnumNode {
        EnumNode::Socket(self)
    }
    fn denumerate(val: EnumNode) -> Self
    where
        Self: Sized,
    {
        if let EnumNode::Socket(s) = val {
            s
        } else {
            panic!()
        }
    }
}

impl Generable for Socket {
    type Additional = SocketInit;

    fn new(id: NodeID, uuid: Uuid5, additional: Option<Self::Additional>) -> Self
    where
        Self: Sized,
    {
        let mut s = Socket {
            db_id: id,
            uuid,
            class: SocketClass::Unknown,
            path: String::new(),
            ip: String::new(),
            port: 0,
        };
        if let Some(add) = additional {
            s.class = add.class;
            s.path = add.path;
            s.ip = add.ip;
            s.port = add.port;
        }
        s
    }
}

impl HasUUID for Socket {
    fn get_uuid(&self) -> Uuid5 {
        self.uuid
    }
}
