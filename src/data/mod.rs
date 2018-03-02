mod nodeid;
mod gen_node;
pub mod node_types;

use std::collections::HashMap;
use packstream::values::Value;
use uuid::Uuid5;
use value_as::CastValue;

pub use self::nodeid::NodeID;
pub use self::gen_node::GenNode;

use self::node_types::EnumNode;

pub trait Enumerable {
    fn enumerate(self) -> EnumNode;
    fn denumerate(val: EnumNode) -> Self
    where
        Self: Sized;
}

pub trait HasID {
    fn get_db_id(&self) -> NodeID;
}

pub trait HasUUID {
    fn get_uuid(&self) -> Uuid5;
}

pub trait ToDB: HasID {
    fn to_db(&self) -> Value;
    fn get_labels(&self) -> Value;
}

pub trait Generable: HasID + HasUUID {
    fn new(id: NodeID, uuid: Uuid5, additional: Option<HashMap<&'static str, Value>>) -> Self
    where
        Self: Sized;
}

pub enum Edge {
    Child(NodeID),
    Next(NodeID),
}

impl Edge {
    pub fn to_db(&self) -> Value {
        let mut prop: HashMap<&'static str, Value> = HashMap::new();
        match *self {
            Edge::Child(n) => {
                prop.insert("id", n.into());
                prop.insert("class", "child".into());
            }
            Edge::Next(n) => {
                prop.insert("id", n.into());
                prop.insert("class", "next".into());
            }
        }
        prop.into()
    }

    pub fn from_db(val: Value) -> Result<Edge, &'static str> {
        match val {
            Value::Structure {
                signature,
                mut fields,
            } => {
                assert_eq!(signature, 0x52);
                assert_eq!(fields.len(), 5);
                let dest_id = NodeID::new(fields
                    .remove(2)
                    .as_int()
                    .ok_or("DestID field is not an Integer")?);
                let class = fields
                    .remove(3)
                    .as_map()
                    .and_then(|mut i| i.remove("class"))
                    .and_then(|i| (&i).as_string())
                    .ok_or(
                        "Edge class property missing, not a string or properties field not a map",
                    )?;
                match &class[..] {
                    "child" => Ok(Edge::Child(dest_id)),
                    "next" => Ok(Edge::Next(dest_id)),
                    _ => Err("Invalid edge class"),
                }
            }
            _ => Err("Value is not an edge"),
        }
    }
}
