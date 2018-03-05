mod process;
mod file;
mod editsession;

use packstream::values::Value;

pub use self::process::Process;
pub use self::file::File;
pub use self::editsession::EditSession;

use super::gen_node::GenNode;
use super::{HasID, HasUUID, NodeID, ToDB};
use uuid::Uuid5;

#[derive(Debug)]
pub enum EnumNode {
    Proc(Process),
    File(File),
    EditSession(EditSession),
}

impl EnumNode {
    pub fn from_db(val: Value) -> Result<EnumNode, &'static str> {
        let g = GenNode::from_db(val)?;
        if g.labs.contains(&String::from("Process")) {
            Ok(EnumNode::Proc(Process::from_props(g.props)?))
        } else if g.labs.contains(&String::from("File")) {
            Ok(EnumNode::File(File::from_props(g.props)?))
        } else if g.labs.contains(&String::from("EditSession")) {
            Ok(EnumNode::EditSession(EditSession::from_props(g.props)?))
        } else {
            Err("Node doesn't match any known type.")
        }
    }
}

impl HasID for EnumNode {
    fn get_db_id(&self) -> NodeID {
        match *self {
            EnumNode::Proc(ref p) => p.get_db_id(),
            EnumNode::File(ref f) => f.get_db_id(),
            EnumNode::EditSession(ref e) => e.get_db_id(),
        }
    }
}

impl HasUUID for EnumNode {
    fn get_uuid(&self) -> Uuid5 {
        match *self {
            EnumNode::Proc(ref p) => p.get_uuid(),
            EnumNode::File(ref f) => f.get_uuid(),
            EnumNode::EditSession(ref e) => e.get_uuid(),
        }
    }
}

impl ToDB for EnumNode {
    fn to_db(&self) -> Value {
        match *self {
            EnumNode::Proc(ref p) => p.to_db(),
            EnumNode::File(ref f) => f.to_db(),
            EnumNode::EditSession(ref e) => e.to_db(),
        }
    }
    fn get_labels(&self) -> Value {
        match *self {
            EnumNode::Proc(ref p) => p.get_labels(),
            EnumNode::File(ref f) => f.get_labels(),
            EnumNode::EditSession(ref e) => e.get_labels(),
        }
    }
}
