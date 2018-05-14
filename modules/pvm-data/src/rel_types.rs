use {id::ID, HasDst, HasID, HasSrc, RelGenerable};

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

    fn new(id: ID, src: ID, dst: ID, init: <Self as RelGenerable>::Init) -> Self
    where
        Self: Sized,
    {
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
pub enum Rel {
    Inf(Inf),
}

macro_rules! rel_trait {
    ($TR: ty,
     $($F:ident() -> $T: ty),*) => {
        impl $TR for Rel {
            $(fn $F(&self) -> $T {
                match self {
                    Rel::Inf(i) => i.$F(),
                }
            })*
        }
    }
}

rel_trait!(HasID, get_db_id() -> ID);
rel_trait!(HasSrc, get_src() -> ID);
rel_trait!(HasDst, get_dst() -> ID);
