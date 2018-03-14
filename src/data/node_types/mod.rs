mod process;
mod file;
mod editsession;
mod socket;

use packstream::values::Value;

pub use self::process::{Process, ProcessInit};
pub use self::file::{File, FileInit};
pub use self::editsession::{EditInit, EditSession};
pub use self::socket::{Socket, SocketClass, SocketInit};

use super::gen_node::GenNode;
use super::{HasID, HasUUID, NodeID, ToDB};
use uuid::Uuid5;

#[derive(Debug)]
pub enum EnumNode {
    Proc(Process),
    File(File),
    EditSession(EditSession),
    Socket(Socket),
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
        } else if g.labs.contains(&String::from("Socket")) {
            Ok(EnumNode::Socket(Socket::from_props(g.props)?))
        } else {
            Err("Node doesn't match any known type.")
        }
    }
}

macro_rules! enumnode_trait {
    ($TR: ty,
     $($F:ident() -> $T: ty),*) => {
        impl $TR for EnumNode {
            $(fn $F(&self) -> $T {
                match *self {
                    EnumNode::Proc(ref p) => p.$F(),
                    EnumNode::EditSession(ref e) => e.$F(),
                    EnumNode::File(ref f) => f.$F(),
                    EnumNode::Socket(ref s) => s.$F(),
                }
            })*
        }
    }
}

enumnode_trait!(HasID,
                get_db_id() -> NodeID);

enumnode_trait!(HasUUID,
                get_uuid() -> Uuid5);

enumnode_trait!(ToDB,
                to_db() -> Value,
                get_labels() -> Value);

