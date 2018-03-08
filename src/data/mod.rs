mod nodeid;
mod gen_node;
pub mod node_types;
mod edge;

use packstream::values::Value;
use uuid::Uuid5;

pub use self::nodeid::NodeID;
pub use self::gen_node::GenNode;
pub use self::edge::Edge;

use self::node_types::EnumNode;

pub trait Enumerable {
    fn enumerate(self) -> EnumNode;
    fn denumerate(val: EnumNode) -> Self
    where
        Self: Sized;
}

pub trait HasID {
    fn get_db_id(&self) -> NodeID;
}

pub trait HasUUID {
    fn get_uuid(&self) -> Uuid5;
}

pub trait ToDB: HasID {
    fn to_db(&self) -> Value;
    fn get_labels(&self) -> Value;
}

pub trait Generable: HasID + HasUUID {
    type Additional;

    fn new(id: NodeID, uuid: Uuid5, additional: Option<Self::Additional>) -> Self
    where
        Self: Sized;
}
