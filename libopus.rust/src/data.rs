use std::collections::HashMap;

use packstream::values::{Value, ValueCast};

#[derive(Debug)]
pub struct NodeID(pub u64);

#[derive(Debug)]
pub enum Node {
    Process(ProcessNode),
}

impl Node {
    pub fn from_value(rec: Value) -> Node {
        match rec {
            Value::Structure { signature, fields } => {
                assert!(signature == 0x4E);
                assert!(fields.len() == 3);
                let labs = match fields[1] {
                    Value::List(ref l) => l,
                    _ => panic!(),
                };
                assert!(labs.len() == 1);
                let props = match fields[2] {
                    Value::Map(ref m) => m,
                    _ => panic!(),
                };
                match labs[0] {
                    Value::String(ref s) => {
                        if s == "Process" {
                            Node::Process(ProcessNode::from_props(props).unwrap())
                        } else {
                            panic!()
                        }
                    }
                    _ => panic!(),
                }
            }
            _ => panic!(),
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
        let db_id = match props.get("db_id") {
            Some(v) => {
                match v {
                    &Value::Integer(i) => ::data::NodeID(i as u64),
                    _ => return Err("db_id property is not an Integer"),
                }
            }
            None => return Err("Missing db_id property"),
        };
        let cmdline = match props.get("cmdline") {
            Some(v) => {
                match v {
                    &Value::String(ref s) => s.clone(),
                    _ => return Err("cmdline property is not a String"),
                }
            }
            None => return Err("Missing cmdline property"),
        };
        let uuid = match props.get("uuid") {
            Some(v) => {
                match v {
                    &Value::String(ref s) => s.clone(),
                    _ => return Err("uuid property is not a String"),
                }
            }
            None => return Err("Missing uuid property"),
        };
        let pid = match props.get("pid") {
            Some(v) => {
                match v {
                    &Value::Integer(i) => i as i32,
                    _ => return Err("pid property is not an Integer"),
                }
            }
            None => return Err("Missing pid property"),
        };
        let thin = match props.get("thin") {
            Some(v) => {
                match v {
                    &Value::Boolean(b) => b,
                    _ => return Err("thin property is not a bool"),
                }
            }
            None => return Err("Missing thin property"),
        };
        Ok(ProcessNode {
            db_id: db_id,
            cmdline: cmdline,
            uuid: uuid,
            pid: pid,
            thin: thin,
            rel: Vec::new(),
        })
    }
}

#[derive(Debug)]
pub enum Edge {
    Child(NodeID),
    Next(NodeID),
}
