mod editsession;
mod file;
mod pipe;
mod process;
mod socket;

use std::collections::HashMap;

use neo4j::Value;

pub use self::{editsession::{EditInit, EditSession}, file::{File, FileInit},
               pipe::{Pipe, PipeInit}, process::{Process, ProcessInit},
               socket::{Socket, SocketClass, SocketInit}};

use super::{HasID, HasUUID, NodeID, ToDB, gen_node::GenNode};
use uuid::Uuid5;

#[derive(Debug)]
pub enum EnumNode {
    Proc(Process),
    File(File),
    Pipe(Pipe),
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
        } else if g.labs.contains(&String::from("Pipe")) {
            Ok(EnumNode::Pipe(Pipe::from_props(g.props)?))
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
                    EnumNode::Pipe(ref p) => p.$F(),
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
                get_labels() -> Vec<&'static str>,
                get_props() -> HashMap<&'static str, Value>);
