mod editsession;
mod file;
mod name;
mod pipe;
mod process;
mod ptty;
mod socket;

pub use self::{
    editsession::EditSession, file::{File, FileContainer}, name::{Name, NameNode},
    pipe::{Pipe, PipeInit}, process::{Process, ProcessInit}, ptty::Ptty,
    socket::{Socket, SocketClass, SocketInit},
};

use super::{Denumerate, Enumerable, HasID, HasUUID, ID};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum Node {
    Data(DataNode),
    Name(NameNode),
}

impl HasID for Node {
    fn get_db_id(&self) -> ID {
        match *self {
            Node::Data(ref d) => d.get_db_id(),
            Node::Name(ref n) => n.get_db_id(),
        }
    }
}

impl Enumerable for Node {
    type Target = Node;
    fn enumerate(self) -> Node {
        self
    }
}

impl Enumerable for NameNode {
    type Target = Node;
    fn enumerate(self) -> Node {
        Node::Name(self)
    }
}

impl Enumerable for DataNode {
    type Target = Node;
    fn enumerate(self) -> Node {
        Node::Data(self)
    }
}

#[derive(Clone, Debug)]
pub enum DataNode {
    Proc(Process),
    File(File),
    FileCont(FileContainer),
    Pipe(Pipe),
    EditSession(EditSession),
    Socket(Socket),
    Ptty(Ptty),
}

macro_rules! enumnode_trait {
    ($TR: ty,
     $($F:ident() -> $T: ty),*) => {
        impl $TR for DataNode {
            $(fn $F(&self) -> $T {
                match self {
                    DataNode::Proc(p) => p.$F(),
                    DataNode::Pipe(p) => p.$F(),
                    DataNode::EditSession(e) => e.$F(),
                    DataNode::File(f) => f.$F(),
                    DataNode::FileCont(f) => f.$F(),
                    DataNode::Socket(s) => s.$F(),
                    DataNode::Ptty(p) => p.$F(),
                }
            })*
        }
    }
}

enumnode_trait!(HasID,
                get_db_id() -> ID);

enumnode_trait!(HasUUID,
                get_uuid() -> Uuid);

macro_rules! enum_denum {
    ($V:path, $T:ty) => {
        impl Enumerable for $T {
            type Target = DataNode;
            fn enumerate(self) -> Self::Target {
                $V(self)
            }
        }

        impl Denumerate for $T {
            fn denumerate(val: &DataNode) -> &Self {
                if let $V(ref ed) = *val {
                    ed
                } else {
                    panic!("{:?} is not an {}", val, stringify!($T))
                }
            }
            fn denumerate_mut(val: &mut DataNode) -> &mut Self {
                if let $V(ref mut ed) = *val {
                    ed
                } else {
                    panic!("{:?} is not an {}", val, stringify!($T))
                }
            }
        }
    };
}

enum_denum!(DataNode::EditSession, EditSession);
enum_denum!(DataNode::File, File);
enum_denum!(DataNode::FileCont, FileContainer);
enum_denum!(DataNode::Pipe, Pipe);
enum_denum!(DataNode::Proc, Process);
enum_denum!(DataNode::Socket, Socket);
enum_denum!(DataNode::Ptty, Ptty);
