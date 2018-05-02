extern crate uuid;

mod id;
pub mod node_types;

use uuid::Uuid;

pub use self::id::ID;

use self::node_types::EnumNode;

pub trait Enumerable {
    fn enumerate(self) -> EnumNode;
}

pub trait Denumerate {
    fn denumerate(val: &EnumNode) -> &Self;
    fn denumerate_mut(val: &mut EnumNode) -> &mut Self;
}

pub trait HasID {
    fn get_db_id(&self) -> ID;
}

pub trait HasUUID {
    fn get_uuid(&self) -> Uuid;
}

pub trait Generable: HasID + HasUUID {
    type Init;

    fn new(id: ID, uuid: Uuid, init: Option<Self::Init>) -> Self
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

#[derive(Clone, Copy, Debug)]
pub enum PVMOps {
    Source,
    Sink,
    Connect,
    Version,
}

#[derive(Clone, Debug)]
pub enum Rel {
    Inf {
        id: ID,
        src: ID,
        dst: ID,
        pvm_op: PVMOps,
        generating_call: String,
        byte_count: u64,
    },
}

impl HasID for Rel {
    fn get_db_id(&self) -> ID {
        match *self {
            Rel::Inf { id, .. } => id,
        }
    }
}
