pub mod node_types;
mod nodeid;

use std::collections::HashMap;

use neo4j::Value;
use uuid::Uuid5;

pub use self::nodeid::NodeID;

use self::node_types::EnumNode;

pub trait Enumerable {
    fn enumerate(self) -> EnumNode;
}

pub trait Denumerate {
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
    type Init;

    fn new(id: NodeID, uuid: Uuid5, init: Option<Self::Init>) -> Self
    where
        Self: Sized;
}

impl<'a, T> Enumerable for &'a T
where
    T: Enumerable + Clone,
{
    fn enumerate(self) -> EnumNode {
        <T as Enumerable>::enumerate((*self).clone())
    }
}

impl<'a, T> Enumerable for &'a mut T
where
    T: Enumerable + Clone,
{
    fn enumerate(self) -> EnumNode {
        <T as Enumerable>::enumerate((*self).clone())
    }
}

impl Enumerable for EnumNode {
    fn enumerate(self) -> EnumNode {
        self
    }
}
