mod editsession;
mod file;
mod pipe;
mod process;
mod ptty;
mod socket;

pub use self::{editsession::{EditInit, EditSession},
               file::{File, FileInit},
               pipe::{Pipe, PipeInit},
               process::{Process, ProcessInit},
               ptty::{Ptty, PttyInit},
               socket::{Socket, SocketClass, SocketInit}};

use super::{Denumerate, Enumerable, HasID, HasUUID, ID};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum EnumNode {
    Proc(Process),
    File(File),
    Pipe(Pipe),
    EditSession(EditSession),
    Socket(Socket),
    Ptty(Ptty),
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
                    EnumNode::Ptty(ref p) => p.$F(),
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
    };
}

enum_denum!(EnumNode::EditSession, EditSession);
enum_denum!(EnumNode::File, File);
enum_denum!(EnumNode::Pipe, Pipe);
enum_denum!(EnumNode::Proc, Process);
enum_denum!(EnumNode::Socket, Socket);
enum_denum!(EnumNode::Ptty, Ptty);
