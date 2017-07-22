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
    pub fn from_value(rec: Value) -> Node {
        match rec {
            Value::Structure { signature, fields } => {
                assert!(signature == 0x4E);
                assert!(fields.len() == 3);
                let id = match fields[0] {
                    Value::Integer(ref i) => i,
                    _ => panic!(),
                };
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
        let db_id = ::data::NodeID(props
            .get("db_id")
            .ok_or("Missing db_id property")?
            .as_u64()
            .ok_or("db_id property is not an Integer")?);
        let cmdline = props
            .get("cmdline")
            .ok_or("Missing cmdline property")?
            .as_string()
            .ok_or("cmdline property is not a String")?;
        let uuid = props
            .get("uuid")
            .ok_or("Missing uuid property")?
            .as_string()
            .ok_or("uuid property is not a String")?;
        let pid = props
            .get("pid")
            .ok_or("Missing pid property")?
            .as_i32()
            .ok_or("pid property is not an Integer")?;
        let thin = props
            .get("thin")
            .ok_or("Missing thin property")?
            .as_bool()
            .ok_or("thin property is not a bool")?;
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
