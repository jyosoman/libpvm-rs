mod nodeid;
mod gen_node;
pub mod node_types;
mod edge;


use std::collections::HashMap;

use neo4j::Value;
use uuid::Uuid5;

pub use self::nodeid::NodeID;
pub use self::gen_node::GenNode;
pub use self::edge::Edge;

use self::node_types::EnumNode;

pub trait Enumerable {
    fn enumerate(self) -> EnumNode;
    fn denumerate(val: &EnumNode) -> &Self;
    fn denumerate_mut(val: &mut EnumNode) -> &mut Self;
}

pub trait HasID {
    fn get_db_id(&self) -> NodeID;
}

pub trait HasUUID {
    fn get_uuid(&self) -> Uuid5;
}

pub trait ToDB: HasID {
    fn get_labels(&self) -> Vec<&'static str>;
    fn get_props(&self) -> HashMap<&'static str, Value>;
    fn to_db(&self) -> (NodeID, Vec<&'static str>, HashMap<&'static str, Value>) {
        let mut props = self.get_props();
        props.insert("db_id", self.get_db_id().into());
        (self.get_db_id(), self.get_labels(), props)
    }
}

pub trait Generable: HasID + HasUUID {
    type Additional;

    fn new(id: NodeID, uuid: Uuid5, additional: Option<Self::Additional>) -> Self
    where
        Self: Sized;
}
