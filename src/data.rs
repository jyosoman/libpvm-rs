use std::collections::HashMap;

use packstream::values::Value;

use uuid::Uuid5;
use value_as::CastValue;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct NodeID(i64);

impl NodeID {
    pub fn new(val: i64) -> NodeID {
        NodeID(val)
    }
}

impl From<NodeID> for Value {
    fn from(val: NodeID) -> Self {
        Value::Integer(val.0 as i64)
    }
}

pub struct GenNode {
    pub id: u64,
    pub labs: Vec<String>,
    pub props: HashMap<String, Value>,
}

impl GenNode {
    pub fn from_db(val: Value) -> Result<GenNode, &'static str> {
        match val {
            Value::Structure {
                signature,
                mut fields,
            } => {
                if signature != 0x4E {
                    return Err("Structure has incorrect signature");
                }
                if fields.len() != 3 {
                    return Err("Node structure has incorrect number of fields");
                }
                let id = fields
                    .remove(0)
                    .as_int()
                    .ok_or("id field is not an integer")?;
                let labs = fields
                    .remove(0)
                    .as_vec()
                    .ok_or("labels field is not a list")?
                    .iter()
                    .map(|i| i.as_string().unwrap())
                    .collect();
                let props = fields
                    .remove(0)
                    .as_map()
                    .ok_or("properties field is not a map")?;
                Ok(GenNode {
                    id,
                    labs,
                    props,
                })
            }
            _ => Err("Is not a node value."),
        }
    }
    pub fn decompose(self) -> (u64, Vec<String>, HashMap<String, Value>) {
        (self.id, self.labs, self.props)
    }
}

pub enum MultiNode {
    Proc(ProcessNode),
}

pub trait Node {
    fn to_db(&self) -> Value;
    fn get_labels(&self) -> Value;
    fn get_db_id(&self) -> NodeID;
}

pub struct ProcessNode {
    pub db_id: NodeID,
    pub uuid: Uuid5,
    pub pid: i32,
    pub cmdline: String,
    pub thin: bool,
}

impl MultiNode {
    pub fn from_db(val: Value) -> Result<MultiNode, &'static str> {
        let (_id, labels, props) = GenNode::from_db(val)?.decompose();
        if labels.contains(&String::from("Process")) {
            Ok(MultiNode::Proc(ProcessNode::from_props(props)?))
        } else{
            Err("Node doesn't match any known type.")
        }
    }
}

impl ProcessNode {
    pub fn from_props(props: HashMap<String, Value>) -> Result<Self, &'static str> {
        Ok(ProcessNode {
            db_id: NodeID::new(props
                .get("db_id")
                .and_then(Value::as_int)
                .ok_or("db_id property is missing or not an Integer")?),
            uuid: props
                .get("uuid")
                .and_then(Value::as_uuid5)
                .ok_or("uuid property is missing or not a UUID5")?,
            cmdline: props
                .get("cmdline")
                .and_then(Value::as_string)
                .ok_or("cmdline property is missing or not a String")?,
            pid: props
                .get("pid")
                .and_then(Value::as_int)
                .ok_or("pid property is missing or not an Integer")?,
            thin: props
                .get("thin")
                .and_then(Value::as_bool)
                .ok_or("thin property is missing or not a bool")?,
        })
    }
}

impl Node for ProcessNode {
    fn to_db(&self) -> Value {
        let mut props: HashMap<&'static str, Value> = HashMap::new();
        props.insert("db_id", self.db_id.into());
        props.insert("uuid", self.uuid.into());
        props.insert("cmdline", self.cmdline.clone().into());
        props.insert("pid", self.pid.into());
        props.insert("thin", self.thin.into());
        props.into()
    }

    fn get_labels(&self) -> Value {
        vec!["Node", "Process"].into()
    }

    fn get_db_id(&self) -> NodeID {
        self.db_id
    }
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
                let dest_id = NodeID(fields
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
