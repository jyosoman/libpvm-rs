use std::{
    collections::{HashMap, HashSet}, fmt::{Display, Formatter, Result as FMTResult},
    sync::{
        atomic::{AtomicUsize, Ordering}, mpsc::SyncSender,
    },
};

use data::{
    node_types::{EditInit, EditSession, EnumNode, File, FileInit}, Enumerable, Generable, HasID,
    rel_types::{Inf, InfInit, PVMOps, Rel},
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

pub type NodeGuard = Loan<ID, Box<EnumNode>>;
pub type RelGuard = Loan<(&'static str, ID, ID), Rel>;

pub struct PVM {
    db: DB,
    uuid_cache: HashMap<Uuid, ID>,
    node_cache: LendingLibrary<ID, Box<EnumNode>>,
    rel_cache: LendingLibrary<(&'static str, ID, ID), Rel>,
    id_counter: AtomicUsize,
    open_cache: HashMap<Uuid, HashSet<Uuid>>,
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
        ID::new(self.id_counter.fetch_add(1, Ordering::Relaxed) as i64)
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

    pub fn source(&mut self, act: &EnumNode, ent: &EnumNode) -> RelGuard {
        self._inf(ent, act, PVMOps::Source)
    }

    pub fn sink(&mut self, act: &EnumNode, ent: &EnumNode) -> RelGuard {
        match ent {
            EnumNode::File(fref) => {
                let f = self.add::<File>(
                    fref.get_uuid(),
                    Some(FileInit {
                        name: fref.name.clone(),
                    }),
                );
                self._inf(fref, &**f, PVMOps::Version);
                self._inf(act, &**f, PVMOps::Sink)
            }
            _ => self._inf(act, ent, PVMOps::Sink),
        }
    }

    pub fn sinkstart(&mut self, act: &EnumNode, ent: &EnumNode) -> RelGuard {
        match ent {
            EnumNode::File(fref) => {
                let es = self.add::<EditSession>(
                    fref.get_uuid(),
                    Some(EditInit {
                        name: fref.name.clone(),
                    }),
                );
                self.open_cache
                    .insert(fref.get_uuid(), hashset!(act.get_uuid()));
                self._inf(fref, &**es, PVMOps::Version);
                self._inf(act, &**es, PVMOps::Sink)
            }
            EnumNode::EditSession(eref) => {
                self.open_cache
                    .get_mut(&eref.get_uuid())
                    .unwrap()
                    .insert(act.get_uuid());
                self._inf(act, eref, PVMOps::Sink)
            }
            _ => self._inf(act, ent, PVMOps::Sink),
        }
    }

    pub fn sinkend(&mut self, act: &EnumNode, ent: &EnumNode) {
        if let EnumNode::EditSession(ref eref) = *ent {
            self.open_cache
                .get_mut(&eref.get_uuid())
                .unwrap()
                .remove(&act.get_uuid());
            if self.open_cache[&eref.get_uuid()].is_empty() {
                let f = self.add::<File>(
                    eref.get_uuid(),
                    Some(FileInit {
                        name: eref.name.clone(),
                    }),
                );
                self._inf(eref, &**f, PVMOps::Version);
            }
        }
    }

    pub fn name(&mut self, obj: &mut EnumNode, name: String) {
        match obj {
            EnumNode::File(fref) => {
                if fref.name == "" {
                    fref.name = name;
                    self.db.update_node(fref);
                }
            }
            EnumNode::EditSession(eref) => {
                if eref.name == "" {
                    eref.name = name;
                    self.db.update_node(eref);
                }
            }
            EnumNode::Ptty(pref) => {
                if pref.name == "" {
                    pref.name = name;
                    self.db.update_node(pref);
                }
            }
            _ => {}
        }
    }

    pub fn prop_node(&mut self, ent: &EnumNode) {
        self.db.update_node(ent)
    }

    pub fn prop_rel(&mut self, ent: &Rel) {
        self.db.update_rel(ent)
    }

    pub fn connect(&mut self, first: &EnumNode, second: &EnumNode, dir: ConnectDir) {
        self._inf(first, second, PVMOps::Connect);
        if let ConnectDir::BiDirectional = dir {
            self._inf(second, first, PVMOps::Connect);
        }
    }

    pub fn shutdown(self) {}
}
