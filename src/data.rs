use std::collections::HashMap;

use packstream::values::Value;

use uuid::Uuid5;
use value_as::CastValue;

#[derive(Debug, Copy, Clone)]
pub struct NodeID(u64);

impl From<NodeID> for Value {
    fn from(val: NodeID) -> Self {
        Value::Integer(val.0 as i64)
    }
}

#[derive(Debug)]
pub enum Node {
    Process(ProcessNode),
}

impl Node {
    pub fn from_value(node: Value) -> Result<Node, &'static str> {
        let gen_n = GenNode::from_value(node)?;
        if gen_n.labs.len() != 1 {
            return Err("Node has more than one label");
        }
        match &gen_n.labs[0][..] {
            "Process" => Ok(Node::Process(ProcessNode::from_props(gen_n.props)?)),
            _ => Err("Unrecognised node label"),
        }
    }

    pub fn get_props(&self) -> HashMap<&str, Value> {
        match *self {
            Node::Process(ref p) => p.get_props(),
        }
    }
}

#[derive(Debug)]
pub struct GenNode {
    pub id: u64,
    pub labs: Vec<String>,
    pub props: HashMap<String, Value>,
}

impl GenNode {
    pub fn from_value(val: Value) -> Result<GenNode, &'static str> {
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
                    id: id,
                    labs: labs,
                    props: props,
                })
            }
            _ => Err("Is not a node value."),
        }
    }
}

#[derive(Debug)]
pub struct ProcessNode {
    pub db_id: NodeID,
    pub uuid: Uuid5,
    pub cmdline: String,
    pub pid: i32,
    pub thin: bool,
}

impl ProcessNode {
    pub fn get_props(&self) -> HashMap<&str, Value> {
        let mut props = HashMap::new();
        props.insert("db_id", self.db_id.into());
        props.insert("uuid", self.uuid.into());
        props.insert("cmdline", self.cmdline.clone().into());
        props.insert("pid", self.pid.into());
        props.insert("thin", self.thin.into());
        props
    }

    pub fn from_props(props: HashMap<String, Value>) -> Result<ProcessNode, &'static str> {
        Ok(ProcessNode {
            db_id: ::data::NodeID(props
                .get("db_id")
                .and_then(Value::as_int)
                .ok_or("db_id property is missing or not an Integer")?),
            cmdline: props
                .get("cmdline")
                .and_then(Value::as_string)
                .ok_or("cmdline property is missing or not a String")?,
            uuid: props
                .get("uuid")
                .and_then(Value::as_uuid5)
                .ok_or("uuid property is missing or not a UUID5")?,
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

#[derive(Debug)]
pub enum Edge {
    Child(NodeID),
    Next(NodeID),
}

impl Edge {
    pub fn get_props(&self) -> Value {
        let mut prop = HashMap::new();
        match *self {
            Edge::Child(n) => {
                prop.insert("id".to_string(), n.into());
                prop.insert("class".to_string(), "child".into());
            }
            Edge::Next(n) => {
                prop.insert("id".to_string(), n.into());
                prop.insert("class".to_string(), "next".into());
            }
        }
        Value::Map(prop)
    }

    pub fn from_value(val: Value) -> Result<Edge, &'static str> {
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
