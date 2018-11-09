use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter, Result as FMTResult},
    mem,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::SyncSender,
    },
};

use data::{
    node_types::{
        ConcreteType, ContextType, CtxNode, DataNode, Name, NameNode, PVMDataType, PVMDataType::*,
        SchemaNode,
    },
    rel_types::{Inf, InfInit, Named, NamedInit, PVMOps, Rel},
    Denumerate, Enumerable, HasID, MetaStore, RelGenerable, ID,
};
use views::DBTr;

use either::Either;
use lending_library::{LendingLibrary, Loan};
use uuid::Uuid;

use super::db::DB;

pub enum PVMError {
    MissingField { evt: String, field: &'static str },
}

impl Display for PVMError {
    fn fmt(&self, f: &mut Formatter) -> FMTResult {
        match self {
            PVMError::MissingField { evt, field } => {
                write!(f, "Event {} missing needed field {}", evt, field)
            }
        }
    }
}

#[derive(Debug)]
pub struct IDCounter {
    store: AtomicUsize,
}

impl IDCounter {
    pub fn new(init: usize) -> Self {
        IDCounter {
            store: AtomicUsize::new(init),
        }
    }

    pub fn get(&self) -> ID {
        ID::new(self.store.fetch_add(1, Ordering::Relaxed) as u64)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ConnectDir {
    Mono,
    BiDirectional,
}

enum CtxStore {
    Node(ID),
    Lazy(&'static ContextType, HashMap<&'static str, String>),
}

pub struct PVM {
    db: DB,
    type_cache: HashSet<&'static ConcreteType>,
    ctx_type_cache: HashSet<&'static ContextType>,
    uuid_cache: HashMap<Uuid, ID>,
    node_cache: LendingLibrary<ID, DataNode>,
    rel_src_dst_cache: HashMap<(&'static str, ID, ID), ID>,
    rel_cache: LendingLibrary<ID, Rel>,
    id: IDCounter,
    open_cache: HashMap<Uuid, HashSet<Uuid>>,
    name_cache: LendingLibrary<Name, NameNode>,
    cont_cache: HashMap<Uuid, ID>,
    cur_ctx: CtxStore,
    pub unparsed_events: HashSet<String>,
}

impl PVM {
    pub fn new(db: SyncSender<DBTr>) -> Self {
        PVM {
            db: DB::create(db),
            type_cache: HashSet::new(),
            ctx_type_cache: HashSet::new(),
            uuid_cache: HashMap::new(),
            node_cache: LendingLibrary::new(),
            rel_src_dst_cache: HashMap::new(),
            rel_cache: LendingLibrary::new(),
            id: IDCounter::new(1),
            open_cache: HashMap::new(),
            name_cache: LendingLibrary::new(),
            cont_cache: HashMap::new(),
            cur_ctx: CtxStore::Node(ID::new(0)),
            unparsed_events: HashSet::new(),
        }
    }

    pub fn new_ctx(&mut self, ty: &'static ContextType, cont: HashMap<&'static str, String>) {
        assert!(self.ctx_type_cache.contains(ty));
        self.cur_ctx = CtxStore::Lazy(ty, cont);
    }

    pub fn ctx(&mut self) -> ID {
        match self.cur_ctx {
            CtxStore::Node(i) => i,
            CtxStore::Lazy(..) => {
                let id = self.id.get();
                let (ty, cont) = match mem::replace(&mut self.cur_ctx, CtxStore::Node(id)) {
                    CtxStore::Lazy(ty, c) => (ty, c),
                    CtxStore::Node(_) => unreachable!(),
                };
                self.db.create_node(CtxNode::new(id, ty, cont).unwrap());
                id
            }
        }
    }

    pub fn release(&mut self, uuid: &Uuid) {
        if let Some(nid) = self.uuid_cache.remove(uuid) {
            self.node_cache.remove(&nid);
        }
    }

    fn _node(&mut self, id: ID) -> Loan<ID, DataNode> {
        self.node_cache.lend(&id).unwrap()
    }

    fn _rel(&mut self, id: ID) -> Loan<ID, Rel> {
        self.rel_cache.lend(&id).unwrap()
    }

    fn _decl_rel<T: RelGenerable + Enumerable<Target = Rel>, S: Fn(ID) -> T::Init>(
        &mut self,
        src: ID,
        dst: ID,
        init: S,
    ) -> ID {
        let triple = (stringify!(T), src, dst);
        if self.rel_src_dst_cache.contains_key(&triple) {
            self.rel_src_dst_cache[&triple]
        } else {
            let id = self.id.get();
            let rel = T::new(id, src, dst, init(self.ctx())).enumerate();
            self.db.create_rel(&rel);
            self.rel_src_dst_cache.insert(triple, id);
            self.rel_cache.insert(id, rel);
            id
        }
    }

    fn _inf(&mut self, src: impl HasID, dst: impl HasID, pvm_op: PVMOps) -> ID {
        self._decl_rel::<Inf, _>(src.get_db_id(), dst.get_db_id(), |ctx| InfInit {
            pvm_op,
            ctx,
            byte_count: 0,
        })
    }

    fn _named(&mut self, src: impl HasID, dst: &NameNode) -> ID {
        self._decl_rel::<Named, _>(src.get_db_id(), dst.get_db_id(), |ctx| NamedInit {
            start: ctx,
            end: ID::new(0),
        })
    }

    pub fn register_data_type(&mut self, ty: &'static ConcreteType) {
        self.type_cache.insert(ty);
        self.db.create_node(SchemaNode::from_data(self.id.get(), ty));
    }

    pub fn register_ctx_type(&mut self, ty: &'static ContextType) {
        self.ctx_type_cache.insert(ty);
        self.db.create_node(SchemaNode::from_ctx(self.id.get(), ty));
    }

    pub fn add(
        &mut self,
        pvm_ty: PVMDataType,
        ty: &'static ConcreteType,
        uuid: Uuid,
        init: Option<MetaStore>,
    ) -> ID {
        assert!(self.type_cache.contains(&ty));
        let id = self.id.get();
        let node = DataNode::new(pvm_ty, ty, id, uuid, self.ctx(), init);
        if let Some(nid) = self.uuid_cache.insert(uuid, id) {
            self.node_cache.remove(&nid);
        }
        self.db.create_node(&node);
        self.node_cache.insert(id, node);
        id
    }

    pub fn declare(
        &mut self,
        ty: &'static ConcreteType,
        uuid: Uuid,
        init: Option<HashMap<&'static str, String>>,
    ) -> ID {
        if !self.uuid_cache.contains_key(&uuid) {
            let init = match init {
                Some(v) => Some(MetaStore::from_map(v, self.ctx(), ty)),
                None => None,
            };
            self.add(ty.pvm_ty, ty, uuid, init)
        } else {
            self.uuid_cache[&uuid]
        }
    }

    fn _version(&mut self, src: &DataNode, choice: Either<Uuid, PVMDataType>) -> ID {
        let ctx = self.ctx();
        let dst = match choice {
            Either::Left(uuid) => {
                let dst_id = self.declare(src.ty(), uuid, None);
                let mut dst = self._node(dst_id);
                dst.meta.merge(&src.meta.snapshot(ctx));
                self.db.update_node(&*dst);
                dst_id
            }
            Either::Right(pvm_ty) => {
                self.add(pvm_ty, src.ty(), src.uuid(), Some(src.meta.snapshot(ctx)))
            }
        };
        self._inf(src, dst, PVMOps::Version);
        dst
    }

    pub fn derive(&mut self, src: ID, dst: Uuid) -> ID {
        let src = self._node(src);
        self._version(&src, Either::Left(dst))
    }

    pub fn source(&mut self, act: ID, ent: ID) -> ID {
        assert_eq!(self._node(act).pvm_ty(), &Actor);
        self._inf(ent, act, PVMOps::Source)
    }

    pub fn source_nbytes<T: Into<i64>>(
        &mut self,
        act: ID,
        ent: ID,
        bytes: T,
    ) -> ID {
        assert_eq!(self._node(act).pvm_ty(), &Actor);
        let id = self.source(act, ent);
        let mut r = self._rel(id);
        Inf::denumerate_mut(&mut r).byte_count += bytes.into();
        self.db.update_rel(&*r);
        id
    }

    pub fn sink(&mut self, act: ID, ent: ID) -> ID {
        let ent = self._node(ent);
        assert_eq!(self._node(act).pvm_ty(), &Actor);
        match ent.pvm_ty() {
            Store => {
                let f = self._version(&ent, Either::Right(Store));
                self._inf(act, f, PVMOps::Sink)
            }
            _ => self._inf(act, &*ent, PVMOps::Sink),
        }
    }

    pub fn sinkstart(&mut self, act: ID, ent: ID) -> ID {
        let act = self._node(act);
        let ent = self._node(ent);
        assert_eq!(act.pvm_ty(), &Actor);
        match ent.pvm_ty() {
            Store => {
                let es = self._version(&ent, Either::Right(EditSession));
                self.open_cache.insert(ent.uuid(), hashset!(act.uuid()));
                self._inf(&*act, es, PVMOps::Sink)
            }
            EditSession => {
                self.open_cache
                    .get_mut(&ent.uuid())
                    .unwrap()
                    .insert(act.uuid());
                self._inf(&*act, &*ent, PVMOps::Sink)
            }
            _ => self._inf(&*act, &*ent, PVMOps::Sink),
        }
    }

    pub fn sinkstart_nbytes<T: Into<i64>>(
        &mut self,
        act: ID,
        ent: ID,
        bytes: T,
    ) -> ID {
        assert_eq!(self._node(act).pvm_ty(), &Actor);
        let id = self.sinkstart(act, ent);
        let mut r = self._rel(id);
        Inf::denumerate_mut(&mut r).byte_count += bytes.into();
        self.db.update_rel(&*r);
        id
    }

    pub fn sinkend(&mut self, act: ID, ent: ID) {
        let ent = self._node(ent);
        let act = self._node(act);
        assert_eq!(act.pvm_ty(), &Actor);
        if let EditSession = ent.pvm_ty() {
            self.open_cache
                .get_mut(&ent.uuid())
                .unwrap()
                .remove(&act.uuid());
            if self.open_cache[&ent.uuid()].is_empty() {
                self._version(&ent, Either::Right(Store));
            }
        }
    }

    fn decl_name(&mut self, name: Name) -> Loan<Name, NameNode> {
        if !self.name_cache.contains_key(&name) {
            let n = NameNode::generate(self.id.get(), name.clone());
            self.db.create_node(&n);
            self.name_cache.insert(name.clone(), n);
        }
        self.name_cache.lend(&name).unwrap()
    }

    fn decl_fcont(&mut self, ent: &DataNode) -> ID {
        if !self.cont_cache.contains_key(&ent.uuid()) {
            let id = self.id.get();
            let node = DataNode::new(StoreCont, ent.ty(), id, ent.uuid(), ID::new(0), None);
            self.cont_cache.insert(ent.uuid(), id);
            self.db.create_node(&node);
            self.node_cache.insert(id, node);
            id
        } else {
            self.cont_cache[&ent.uuid()]
        }
    }

    pub fn name(&mut self, obj: ID, name: Name) -> ID {
        let obj = self._node(obj);
        let n_node = self.decl_name(name);
        match obj.pvm_ty() {
            Store | EditSession => {
                let cont = self.decl_fcont(&obj);
                self._named(cont, &n_node)
            }
            _ => self._named(&*obj, &n_node),
        }
    }

    pub fn unname(&mut self, obj: ID, name: Name) -> ID {
        let id = self.name(obj, name);
        let mut rel = self._rel(id);
        if let Rel::Named(ref mut n_rel) = *rel {
            n_rel.end = self.ctx();
            self.db.update_rel(&*rel);
        }
        id
    }

    pub fn meta<T: ToString + ?Sized>(&mut self, ent: ID, key: &'static str, val: &T) {
        let mut ent = self._node(ent);
        if !ent.ty().props.contains_key(key) {
            panic!("Setting unknown property on concrete type: {:?} does not have a property named {}.", ent.ty(), key);
        }
        let heritable = ent.ty().props[key];
        ent.meta.update(key, val, self.ctx(), heritable);
        self.db.update_node(&*ent);
    }

    pub fn connect(&mut self, first: ID, second: ID, dir: ConnectDir) {
        assert_eq!(self._node(first).pvm_ty(), &Conduit);
        assert_eq!(self._node(second).pvm_ty(), &Conduit);
        self._inf(first, second, PVMOps::Connect);
        if let ConnectDir::BiDirectional = dir {
            self._inf(second, first, PVMOps::Connect);
        }
    }

    pub fn shutdown(self) {}
}
