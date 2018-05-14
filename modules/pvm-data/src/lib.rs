extern crate uuid;

mod id;
pub mod node_types;
pub mod rel_types;

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

pub trait HasSrc {
    fn get_src(&self) -> ID;
}

pub trait HasDst {
    fn get_dst(&self) -> ID;
}

pub trait Generable: HasID + HasUUID {
    type Init;

    fn new(id: ID, uuid: Uuid, init: Option<Self::Init>) -> Self
    where
        Self: Sized;
}

pub trait RelGenerable: HasID + HasSrc + HasDst {
    type Init;

    fn new(id: ID, src: ID, dst: ID, init: Self::Init) -> Self
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
