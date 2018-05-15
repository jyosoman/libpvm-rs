use {HasID, ID};

#[derive(Clone, Debug, Hash, PartialEq)]
pub enum Name {
    Path(String),
    Net(String, u16),
}

#[derive(Clone, Debug)]
pub enum NameNode {
    Path(ID, String),
    Net(ID, String, u16),
}

impl HasID for NameNode {
    fn get_db_id(&self) -> ID {
        match self {
            NameNode::Path(id, _) => *id,
            NameNode::Net(id, _, _) => *id,
        }
    }
}

impl NameNode {
    pub fn generate(id: ID, init: Name) -> Self {
        match init {
            Name::Path(s) => NameNode::Path(id, s),
            Name::Net(s, p) => NameNode::Net(id, s, p),
        }
    }
}
