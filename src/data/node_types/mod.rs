mod editsession;
mod file;
mod pipe;
mod process;
mod socket;

use neo4j::{Node, Value};

pub use self::{editsession::{EditInit, EditSession}, file::{File, FileInit},
               pipe::{Pipe, PipeInit}, process::{Process, ProcessInit},
               socket::{Socket, SocketClass, SocketInit}};

use super::{Enumerable, Denumerate, HasID, HasUUID, NodeID};
use uuid::Uuid5;

#[derive(Clone, Debug)]
pub enum EnumNode {
    Proc(Process),
    File(File),
    Pipe(Pipe),
    EditSession(EditSession),
    Socket(Socket),
}

impl EnumNode {
    pub fn from_db(val: Value) -> Result<EnumNode, &'static str> {
        let g = Node::from_value(val)?;
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

macro_rules! enum_denum {
    ($V:path, $T:ty) => {
        impl Enumerable for $T {
            fn enumerate(self) -> EnumNode {
                $V(self)
            }
        }

        impl Denumerate for $T {
            fn denumerate(val: &EnumNode) -> &Self {
                if let $V(ref ed) = *val {
                    ed
                } else {
                    panic!("{:?} is not an {}", val, stringify!($T))
                }
            }
            fn denumerate_mut(val: &mut EnumNode) -> &mut Self {
                if let $V(ref mut ed) = *val {
                    ed
                } else {
                    panic!("{:?} is not an {}", val, stringify!($T))
                }
            }
        }
    }
}

enum_denum!(EnumNode::EditSession, EditSession);
enum_denum!(EnumNode::File, File);
enum_denum!(EnumNode::Pipe, Pipe);
enum_denum!(EnumNode::Proc, Process);
enum_denum!(EnumNode::Socket, Socket);
