use chrono::{DateTime, Utc};

use {HasDst, HasID, HasSrc, RelGenerable, ID};

#[derive(Clone, Copy, Debug)]
pub enum PVMOps {
    Source,
    Sink,
    Connect,
    Version,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct Inf {
    id: ID,
    src: ID,
    dst: ID,
    pub pvm_op: PVMOps,
    pub generating_call: String,
    pub byte_count: u64,
}

#[derive(Debug)]
pub struct InfInit {
    pub pvm_op: PVMOps,
    pub generating_call: String,
    pub byte_count: u64,
}

impl HasID for Inf {
    fn get_db_id(&self) -> ID {
        self.id
    }
}

impl HasSrc for Inf {
    fn get_src(&self) -> ID {
        self.src
    }
}

impl HasDst for Inf {
    fn get_dst(&self) -> ID {
        self.dst
    }
}

impl RelGenerable for Inf {
    type Init = InfInit;

    fn new(id: ID, src: ID, dst: ID, init: Self::Init) -> Self {
        Inf {
            id,
            src,
            dst,
            pvm_op: init.pvm_op,
            generating_call: init.generating_call,
            byte_count: init.byte_count,
        }
    }
}

impl From<Inf> for Rel {
    fn from(val: Inf) -> Self {
        Rel::Inf(val)
    }
}

#[derive(Clone, Debug)]
pub struct Named {
    id: ID,
    src: ID,
    dst: ID,
    pub generating_call: String,
    pub start: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NamedInit {
    pub generating_call: String,
    pub start: DateTime<Utc>,
}

impl HasID for Named {
    fn get_db_id(&self) -> ID {
        self.id
    }
}

impl HasSrc for Named {
    fn get_src(&self) -> ID {
        self.src
    }
}

impl HasDst for Named {
    fn get_dst(&self) -> ID {
        self.dst
    }
}

impl RelGenerable for Named {
    type Init = NamedInit;

    fn new(id: ID, src: ID, dst: ID, init: Self::Init) -> Self {
        Named {
            id,
            src,
            dst,
            start: init.start,
            generating_call: init.generating_call,
        }
    }
}

impl From<Named> for Rel {
    fn from(val: Named) -> Self {
        Rel::Named(val)
    }
}

#[derive(Clone, Debug)]
pub enum Rel {
    Inf(Inf),
    Named(Named),
}

macro_rules! rel_trait {
    ($TR: ty,
     $($F:ident() -> $T: ty),*) => {
        impl $TR for Rel {
            $(fn $F(&self) -> $T {
                match self {
                    Rel::Inf(i) => i.$F(),
                    Rel::Named(n) => n.$F(),
                }
            })*
        }
    }
}

rel_trait!(HasID, get_db_id() -> ID);
rel_trait!(HasSrc, get_src() -> ID);
rel_trait!(HasDst, get_dst() -> ID);
