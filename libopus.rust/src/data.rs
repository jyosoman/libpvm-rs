#[derive(Debug)]
pub struct NodeID(pub u64);

pub trait Node {}

#[derive(Debug)]
pub struct Process {
    pub db_id: NodeID,
    pub uuid: String,
    pub cmdline: String,
    pub pid: i32,
    pub thin: bool,
    pub rel: Vec<Edge>,
}

impl Node for Process {}

#[derive(Debug)]
pub enum Edge {
    Child(NodeID),
    Next(NodeID),
}
