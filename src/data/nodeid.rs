use packstream::values::Value;

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
