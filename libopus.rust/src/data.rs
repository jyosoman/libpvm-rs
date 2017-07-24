use std::collections::HashMap;

use packstream::values::{Value, ValueCast};

use value_as::CastValue;

#[derive(Debug)]
pub struct NodeID(pub u64);

#[derive(Debug)]
pub enum Node {
    Process(ProcessNode),
}

impl Node {
    pub fn from_value(rec: Value) -> Result<Node, &'static str> {
        match rec {
            Value::Structure { signature, fields } => {
                assert_eq!(signature, 0x4E);
                assert_eq!(fields.len(), 3);
                let id = fields[0].as_i64().unwrap();
                let labs = fields[1].as_vec_ref().unwrap();
                assert_eq!(labs.len(), 1);
                let label = labs[0].as_string().unwrap();
                let props = match fields[2] {
                    Value::Map(ref m) => m,
                    _ => panic!(),
                };
                match &label[..] {
                    "Process" => {
                        match ProcessNode::from_props(props) {
                            Ok(p) => Ok(Node::Process(p)),
                            Err(_) => Err("Failed to parse node from properties"),
                        }
                    }
                    _ => Err("Unrecognised node label"),
                }
            }
            _ => Err("Is not a node value."),
        }
    }

    pub fn get_props(&self) -> HashMap<&str, Value> {
        match self {
            &Node::Process(ref p) => p.get_props(),
        }
    }
}

#[derive(Debug)]
pub struct ProcessNode {
    pub db_id: NodeID,
    pub uuid: String,
    pub cmdline: String,
    pub pid: i32,
    pub thin: bool,
    pub rel: Vec<Edge>,
}

impl ProcessNode {
    pub fn get_props(&self) -> HashMap<&str, Value> {
        let mut props = HashMap::new();
        props.insert("db_id", self.db_id.0.from());
        props.insert("uuid", self.uuid.from());
        props.insert("cmdline", self.cmdline.from());
        props.insert("pid", self.pid.from());
        props.insert("thin", self.thin.from());
        props
    }

    pub fn from_props(props: &HashMap<String, Value>) -> Result<ProcessNode, &'static str> {
        Ok(ProcessNode {
            db_id: ::data::NodeID(props
                .get("db_id")
                .and_then(Value::as_u64)
                .ok_or("db_id property is missing or not an Integer")?),
            cmdline: props
                .get("cmdline")
                .and_then(Value::as_string)
                .ok_or("cmdline property is missing or not a String")?,
            uuid: props
                .get("uuid")
                .and_then(Value::as_string)
                .ok_or("uuid property is missing or not a String")?,
            pid: props
                .get("pid")
                .and_then(Value::as_i32)
                .ok_or("pid property is missing or not an Integer")?,
            thin: props
                .get("thin")
                .and_then(Value::as_bool)
                .ok_or("thin property is missing or not a bool")?,
            rel: Vec::new(),
        })
    }
}

#[derive(Debug)]
pub enum Edge {
    Child(NodeID),
    Next(NodeID),
}
