use neo4j::Value;
use std::fmt::Display;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
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

impl Display for NodeID {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
