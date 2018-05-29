#![feature(never_type)]

extern crate chrono;
extern crate uuid;

mod id;
pub mod node_types;
pub mod rel_types;

use uuid::Uuid;

pub use self::id::ID;

pub trait Enumerable {
    type Target;
    fn enumerate(self) -> Self::Target;
}

pub trait Denumerate: Enumerable {
    fn denumerate(val: &<Self as Enumerable>::Target) -> &Self;
    fn denumerate_mut(val: &mut <Self as Enumerable>::Target) -> &mut Self;
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

pub trait Generable: HasID + HasUUID + Sized {
    type Init;

    fn new(id: ID, uuid: Uuid, init: Option<Self::Init>) -> Self;
}

pub trait RelGenerable: HasID + HasSrc + HasDst + Sized {
    type Init;

    fn new(id: ID, src: ID, dst: ID, init: Self::Init) -> Self;
}

impl<'a, T> Enumerable for &'a T
where
    T: Enumerable + Clone,
{
    type Target = <T as Enumerable>::Target;
    fn enumerate(self) -> Self::Target {
        <T as Enumerable>::enumerate((*self).clone())
    }
}

impl<'a, T> Enumerable for &'a mut T
where
    T: Enumerable + Clone,
{
    type Target = <T as Enumerable>::Target;
    fn enumerate(self) -> Self::Target {
        <T as Enumerable>::enumerate((*self).clone())
    }
}
