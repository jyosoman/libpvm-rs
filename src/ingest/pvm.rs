use std::{
    collections::{HashMap, HashSet}, fmt::{Display, Formatter, Result as FMTResult},
    sync::{
        atomic::{AtomicUsize, Ordering}, mpsc::SyncSender,
    },
};

use data::{
    node_types::{DataNode, EditSession, File, FileContainer, Name, NameNode},
    rel_types::{Inf, InfInit, Named, NamedInit, PVMOps, Rel}, Enumerable, Generable, HasID,
    HasUUID, RelGenerable, ID,
};
use views::DBTr;

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

pub enum ConnectDir {
    Mono,
    BiDirectional,
}

pub type NodeGuard = Loan<ID, Box<DataNode>>;
pub type RelGuard = Loan<(&'static str, ID, ID), Rel>;
type NameGuard = Loan<Name, NameNode>;

pub struct PVM {
    db: DB,
    uuid_cache: HashMap<Uuid, ID>,
    node_cache: LendingLibrary<ID, Box<DataNode>>,
    rel_cache: LendingLibrary<(&'static str, ID, ID), Rel>,
    id_counter: AtomicUsize,
    open_cache: HashMap<Uuid, HashSet<Uuid>>,
    name_cache: LendingLibrary<Name, NameNode>,
    cont_cache: HashMap<Uuid, ID>,
    cur_time: u64,
    cur_evt: String,
    pub unparsed_events: HashSet<String>,
}

impl PVM {
    pub fn new(db: SyncSender<DBTr>) -> Self {
        PVM {
            db: DB::create(db),
            uuid_cache: HashMap::new(),
            node_cache: LendingLibrary::new(),
            rel_cache: LendingLibrary::new(),
            id_counter: AtomicUsize::new(0),
            open_cache: HashMap::new(),
            name_cache: LendingLibrary::new(),
            cont_cache: HashMap::new(),
            cur_time: 0,
            cur_evt: String::new(),
            unparsed_events: HashSet::new(),
        }
    }

    pub fn set_time(&mut self, new_time: u64) {
        self.cur_time = new_time;
    }

    pub fn set_evt(&mut self, new_evt: String) {
        self.cur_evt = new_evt;
    }

    pub fn release(&mut self, uuid: &Uuid) {
        if let Some(nid) = self.uuid_cache.remove(uuid) {
            self.node_cache.remove(&nid);
        }
    }

    fn _nextid(&mut self) -> ID {
        ID::new(self.id_counter.fetch_add(1, Ordering::Relaxed) as u64)
    }

    fn _decl_rel<T: RelGenerable + Into<Rel>>(
        &mut self,
        src: ID,
        dst: ID,
        init: T::Init,
    ) -> RelGuard {
        let triple = (stringify!(T), src, dst);
        if self.rel_cache.contains_key(&triple) {
            self.rel_cache.lend(&triple).unwrap()
        } else {
            let id = self._nextid();
            let rel = T::new(id, src, dst, init).into();
            self.rel_cache.insert(triple, rel);
            let r = self.rel_cache.lend(&triple).unwrap();
            self.db.create_rel(&r);
            r
        }
    }

    fn _inf(&mut self, src: &impl HasID, dst: &impl HasID, pvm_op: PVMOps) -> RelGuard {
        self._decl_rel::<Inf>(
            src.get_db_id(),
            dst.get_db_id(),
            InfInit {
                pvm_op,
                generating_call: self.cur_evt.clone(),
                byte_count: 0,
            },
        )
    }

    fn _named(&mut self, src: &impl HasID, dst: &NameNode) -> RelGuard {
        self._decl_rel::<Named>(
            src.get_db_id(),
            dst.get_db_id(),
            NamedInit {
                start: self.cur_time,
                generating_call: self.cur_evt.clone(),
            },
        )
    }

    pub fn add<T>(&mut self, uuid: Uuid, init: Option<T::Init>) -> NodeGuard
    where
        T: Generable + Enumerable,
    {
        let id = self._nextid();
        let node = Box::new(T::new(id, uuid, init).enumerate());
        if let Some(nid) = self.uuid_cache.insert(uuid, id) {
            self.node_cache.remove(&nid);
        }
        self.node_cache.insert(id, node);
        let n = self.node_cache.lend(&id).unwrap();
        self.db.create_node(&**n);
        n
    }

    pub fn declare<T>(&mut self, uuid: Uuid, init: Option<T::Init>) -> NodeGuard
    where
        T: Generable + Enumerable,
    {
        if !self.uuid_cache.contains_key(&uuid) {
            self.add::<T>(uuid, init)
        } else {
            self.node_cache.lend(&self.uuid_cache[&uuid]).unwrap()
        }
    }

    pub fn source(&mut self, act: &DataNode, ent: &DataNode) -> RelGuard {
        self._inf(ent, act, PVMOps::Source)
    }

    pub fn sink(&mut self, act: &DataNode, ent: &DataNode) -> RelGuard {
        match ent {
            DataNode::File(fref) => {
                let f = self.add::<File>(fref.get_uuid(), None);
                self._inf(fref, &**f, PVMOps::Version);
                self._inf(act, &**f, PVMOps::Sink)
            }
            _ => self._inf(act, ent, PVMOps::Sink),
        }
    }

    pub fn sinkstart(&mut self, act: &DataNode, ent: &DataNode) -> RelGuard {
        match ent {
            DataNode::File(fref) => {
                let es = self.add::<EditSession>(fref.get_uuid(), None);
                self.open_cache
                    .insert(fref.get_uuid(), hashset!(act.get_uuid()));
                self._inf(fref, &**es, PVMOps::Version);
                self._inf(act, &**es, PVMOps::Sink)
            }
            DataNode::EditSession(eref) => {
                self.open_cache
                    .get_mut(&eref.get_uuid())
                    .unwrap()
                    .insert(act.get_uuid());
                self._inf(act, eref, PVMOps::Sink)
            }
            _ => self._inf(act, ent, PVMOps::Sink),
        }
    }

    pub fn sinkend(&mut self, act: &DataNode, ent: &DataNode) {
        if let DataNode::EditSession(eref) = ent {
            self.open_cache
                .get_mut(&eref.get_uuid())
                .unwrap()
                .remove(&act.get_uuid());
            if self.open_cache[&eref.get_uuid()].is_empty() {
                let f = self.add::<File>(eref.get_uuid(), None);
                self._inf(eref, &**f, PVMOps::Version);
            }
        }
    }

    fn decl_name(&mut self, name: Name) -> NameGuard {
        if !self.name_cache.contains_key(&name) {
            let n = NameNode::generate(self._nextid(), name.clone());
            self.db.create_node(&n);
            self.name_cache.insert(name.clone(), n);
        }
        self.name_cache.lend(&name).unwrap()
    }

    fn decl_fcont(&mut self, uuid: Uuid) -> NodeGuard {
        let id = {
            if !self.cont_cache.contains_key(&uuid) {
                let id = self._nextid();
                let node = Box::new(FileContainer::new(id, uuid).enumerate());
                self.cont_cache.insert(uuid, id);
                self.db.create_node(&*node);
                self.node_cache.insert(id, node);
                id
            } else {
                self.cont_cache[&uuid]
            }
        };
        self.node_cache.lend(&id).unwrap()
    }

    pub fn name(&mut self, obj: &DataNode, name: Name) {
        let n_node = self.decl_name(name);
        match obj {
            DataNode::File(f) => {
                let cont = self.decl_fcont(f.get_uuid());
                self._named(&**cont, &n_node);
            }
            DataNode::EditSession(f) => {
                let cont = self.decl_fcont(f.get_uuid());
                self._named(&**cont, &n_node);
            }
            _ => {
                self._named(obj, &n_node);
            }
        }
    }

    pub fn prop_node(&mut self, ent: &DataNode) {
        self.db.update_node(ent)
    }

    pub fn prop_rel(&mut self, ent: &Rel) {
        self.db.update_rel(ent)
    }

    pub fn connect(&mut self, first: &DataNode, second: &DataNode, dir: ConnectDir) {
        self._inf(first, second, PVMOps::Connect);
        if let ConnectDir::BiDirectional = dir {
            self._inf(second, first, PVMOps::Connect);
        }
    }

    pub fn shutdown(self) {}
}
