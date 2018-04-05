use std::fmt::Display;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct NodeID(i64);

impl NodeID {
    pub fn new(val: i64) -> NodeID {
        NodeID(val)
    }
    pub fn inner(self) -> i64 {
        self.0
    }
}

impl Display for NodeID {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
