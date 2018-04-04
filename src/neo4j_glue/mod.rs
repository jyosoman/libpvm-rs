mod cypher_view;
mod neo4j_view;

pub use self::{cypher_view::CypherView, neo4j_view::Neo4JView};

use std::collections::HashMap;

use neo4j::Value;

use data::{NodeID, HasID, HasUUID, node_types::EnumNode};

pub trait ToDB: HasID + HasUUID {
    fn get_labels(&self) -> Vec<&'static str>;
    fn get_props(&self) -> HashMap<&'static str, Value>;
    fn to_db(&self) -> (NodeID, Vec<&'static str>, HashMap<&'static str, Value>) {
        let mut props = self.get_props();
        props.insert("db_id", self.get_db_id().into());
        props.insert("uuid", self.get_uuid().into());
        (self.get_db_id(), self.get_labels(), props)
    }
}

impl ToDB for EnumNode {
    fn get_labels(&self) -> Vec<&'static str> {
        match *self {
            EnumNode::EditSession(_) => vec!["Node", "EditSession"],
            EnumNode::File(_) => vec!["Node", "File"],
            EnumNode::Pipe(_) => vec!["Node", "Pipe"],
            EnumNode::Proc(_) => vec!["Node", "Process"],
            EnumNode::Socket(_) => vec!["Node", "Socket"],
        }
    }
    fn get_props(&self) -> HashMap<&'static str, Value> {
        match *self {
            EnumNode::EditSession(ref e) => hashmap!("name"  => Value::from(e.name.clone())),
            EnumNode::File(ref f) => hashmap!("name"  => Value::from(f.name.clone())),
            EnumNode::Pipe(ref p) => hashmap!("fd"    => Value::from(p.fd)),
            EnumNode::Proc(ref p) => hashmap!("cmdline" => Value::from(p.cmdline.clone()),
                                              "pid"     => Value::from(p.pid),
                                              "thin"    => Value::from(p.thin)),
            EnumNode::Socket(ref s) => hashmap!("class"  => Value::from(s.class as i64),
                                                "path" => Value::from(s.path.clone()),
                                                "ip" => Value::from(s.ip.clone()),
                                                "port" => Value::from(s.port)),
        }
    }
}