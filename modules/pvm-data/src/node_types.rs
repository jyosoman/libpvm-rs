use super::{meta_store::MetaStore, Enumerable, HasID, ID};
use std::{
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
};
use uuid::Uuid;

#[derive(Debug, Eq)]
pub struct ConcreteType {
    pub pvm_ty: PVMDataType,
    pub name: &'static str,
    pub props: HashMap<&'static str, bool>,
}

impl Hash for ConcreteType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        <&'static str as Hash>::hash(&self.name, state)
    }
}

impl PartialEq for ConcreteType {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PVMDataType {
    Actor,
    Store,
    Conduit,
    EditSession,
    Object,
    StoreCont,
}

impl fmt::Display for PVMDataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PVMDataType::Actor => "Actor",
                PVMDataType::Conduit => "Conduit",
                PVMDataType::EditSession => "EditSession",
                PVMDataType::Object => "Object",
                PVMDataType::Store => "Store",
                PVMDataType::StoreCont => "StoreCont",
            }
        )
    }
}

#[derive(Clone, Debug)]
pub struct DataNode {
    pvm_ty: PVMDataType,
    ty: &'static ConcreteType,
    id: ID,
    uuid: Uuid,
    pub meta: MetaStore,
}

impl HasID for DataNode {
    fn get_db_id(&self) -> ID {
        self.id
    }
}

impl DataNode {
    pub fn new(
        pvm_type: PVMDataType,
        ty: &'static ConcreteType,
        id: ID,
        uuid: Uuid,
        meta: Option<MetaStore>,
    ) -> DataNode {
        if pvm_type == PVMDataType::EditSession || pvm_type == PVMDataType::StoreCont {
            if ty.pvm_ty != PVMDataType::Store {
                panic!(
                    "Invalid PVMDataType for given ConcreteType: {:?} cannot be a {:?}.",
                    ty.name, pvm_type
                );
            }
        } else if ty.pvm_ty != pvm_type {
            panic!(
                "Invalid PVMDataType for given ConcreteType: {:?} cannot be a {:?}.",
                ty.name, pvm_type
            );
        }
        DataNode {
            pvm_ty: pvm_type,
            id,
            uuid,
            ty,
            meta: meta.unwrap_or_else(MetaStore::new),
        }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn ty(&self) -> &'static ConcreteType {
        self.ty
    }

    pub fn pvm_ty(&self) -> &PVMDataType {
        &self.pvm_ty
    }
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub enum Name {
    Path(String),
    Net(String, u16),
}

#[derive(Clone, Debug)]
pub enum NameNode {
    Path(ID, String),
    Net(ID, String, u16),
}

impl HasID for NameNode {
    fn get_db_id(&self) -> ID {
        match self {
            NameNode::Path(id, _) => *id,
            NameNode::Net(id, _, _) => *id,
        }
    }
}

impl NameNode {
    pub fn generate(id: ID, init: Name) -> Self {
        match init {
            Name::Path(s) => NameNode::Path(id, s),
            Name::Net(s, p) => NameNode::Net(id, s, p),
        }
    }
}
