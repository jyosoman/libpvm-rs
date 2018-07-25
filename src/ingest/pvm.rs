use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter, Result as FMTResult},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::SyncSender,
    },
};

use data::{
    node_types::{ConcreteType, DataNode, Name, NameNode, PVMDataType, PVMDataType::*},
    rel_types::{Inf, InfInit, Named, NamedInit, PVMOps, Rel},
    Denumerate, Enumerable, HasID, MetaStore, RelGenerable, ID,
};
use views::DBTr;

use chrono::{DateTime, TimeZone, Utc};
use lending_library::{LendingLibrary, Loan};
use uuid::Uuid;

use super::db::DB;

pub enum PVMError {
    MissingField {
        evt: String,
        field: &'static str,
    },
    CannotAssignMeta {
        ty: &'static ConcreteType,
        key: &'static str,
        value: String,
    },
}

impl Display for PVMError {
    fn fmt(&self, f: &mut Formatter) -> FMTResult {
        match self {
            PVMError::MissingField { evt, field } => {
                write!(f, "Event {} missing needed field {}", evt, field)
            }
            PVMError::CannotAssignMeta { ty, key, value } => write!(
                f,
                "Cannot assign {}: {} to an object of type {}",
                key, value, ty.name
            ),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ConnectDir {
    Mono,
    BiDirectional,
}

pub type NodeGuard = Loan<ID, DataNode>;
pub type RelGuard = Loan<(&'static str, ID, ID), Rel>;
type NameGuard = Loan<Name, NameNode>;

pub struct PVM {
    db: DB,
    type_cache: HashSet<&'static ConcreteType>,
    uuid_cache: HashMap<Uuid, ID>,
    node_cache: LendingLibrary<ID, DataNode>,
    rel_cache: LendingLibrary<(&'static str, ID, ID), Rel>,
    id_counter: AtomicUsize,
    open_cache: HashMap<Uuid, HashSet<Uuid>>,
    name_cache: LendingLibrary<Name, NameNode>,
    cont_cache: HashMap<Uuid, ID>,
    cur_time: DateTime<Utc>,
    cur_evt: String,
    pub unparsed_events: HashSet<String>,
}

impl PVM {
    pub fn new(db: SyncSender<DBTr>) -> Self {
        PVM {
            db: DB::create(db),
            type_cache: HashSet::new(),
            uuid_cache: HashMap::new(),
            node_cache: LendingLibrary::new(),
            rel_cache: LendingLibrary::new(),
            id_counter: AtomicUsize::new(0),
            open_cache: HashMap::new(),
            name_cache: LendingLibrary::new(),
            cont_cache: HashMap::new(),
            cur_time: Utc.timestamp(0, 0),
            cur_evt: String::new(),
            unparsed_events: HashSet::new(),
        }
    }

    pub fn set_time(&mut self, new_time: DateTime<Utc>) {
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

    fn _decl_rel<T: RelGenerable + Enumerable<Target = Rel>>(
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
            let rel = T::new(id, src, dst, init).enumerate();
            self.rel_cache.insert(triple, rel);
            let r = self.rel_cache.lend(&triple).unwrap();
            self.db.create_rel(&*r);
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
                end: Utc.timestamp(0, 0),
                generating_call: self.cur_evt.clone(),
            },
        )
    }

    pub fn new_concrete(&mut self, ty: &'static ConcreteType) {
        self.type_cache.insert(ty);
        self.db.new_node_type(ty);
    }

    pub fn add(
        &mut self,
        pvm_ty: PVMDataType,
        ty: &'static ConcreteType,
        uuid: Uuid,
        init: Option<MetaStore>,
    ) -> NodeGuard {
        assert!(self.type_cache.contains(&ty));
        let id = self._nextid();
        let node = DataNode::new(pvm_ty, ty, id, uuid, init);
        if let Some(nid) = self.uuid_cache.insert(uuid, id) {
            self.node_cache.remove(&nid);
        }
        self.node_cache.insert(id, node);
        let n = self.node_cache.lend(&id).unwrap();
        self.db.create_node(&*n);
        n
    }

    pub fn declare(
        &mut self,
        ty: &'static ConcreteType,
        uuid: Uuid,
        init: Option<MetaStore>,
    ) -> NodeGuard {
        if !self.uuid_cache.contains_key(&uuid) {
            self.add(ty.pvm_ty, ty, uuid, init)
        } else {
            self.node_cache.lend(&self.uuid_cache[&uuid]).unwrap()
        }
    }

    pub fn source(&mut self, act: &DataNode, ent: &DataNode) -> RelGuard {
        assert_eq!(act.pvm_ty(), &Actor);
        self._inf(ent, act, PVMOps::Source)
    }

    pub fn source_nbytes<T: Into<i64>>(
        &mut self,
        act: &DataNode,
        ent: &DataNode,
        bytes: T,
    ) -> RelGuard {
        assert_eq!(act.pvm_ty(), &Actor);
        let mut r = self.source(act, ent);
        Inf::denumerate_mut(&mut r).byte_count += bytes.into();
        self.db.update_rel(&*r);
        r
    }

    pub fn sink(&mut self, act: &DataNode, ent: &DataNode) -> RelGuard {
        assert_eq!(act.pvm_ty(), &Actor);
        match ent.pvm_ty() {
            Store => {
                let f = self.add(
                    Store,
                    ent.ty(),
                    ent.uuid(),
                    Some(ent.meta.snapshot(&self.cur_time)),
                );
                self._inf(ent, &*f, PVMOps::Version);
                self._inf(act, &*f, PVMOps::Sink)
            }
            _ => self._inf(act, ent, PVMOps::Sink),
        }
    }

    pub fn sinkstart(&mut self, act: &DataNode, ent: &DataNode) -> RelGuard {
        assert_eq!(act.pvm_ty(), &Actor);
        match ent.pvm_ty() {
            Store => {
                let es = self.add(
                    EditSession,
                    ent.ty(),
                    ent.uuid(),
                    Some(ent.meta.snapshot(&self.cur_time)),
                );
                self.open_cache.insert(ent.uuid(), hashset!(act.uuid()));
                self._inf(ent, &*es, PVMOps::Version);
                self._inf(act, &*es, PVMOps::Sink)
            }
            EditSession => {
                self.open_cache
                    .get_mut(&ent.uuid())
                    .unwrap()
                    .insert(act.uuid());
                self._inf(act, ent, PVMOps::Sink)
            }
            _ => self._inf(act, ent, PVMOps::Sink),
        }
    }

    pub fn sinkstart_nbytes<T: Into<i64>>(
        &mut self,
        act: &DataNode,
        ent: &DataNode,
        bytes: T,
    ) -> RelGuard {
        assert_eq!(act.pvm_ty(), &Actor);
        let mut r = self.sinkstart(act, ent);
        Inf::denumerate_mut(&mut r).byte_count += bytes.into();
        self.db.update_rel(&*r);
        r
    }

    pub fn sinkend(&mut self, act: &DataNode, ent: &DataNode) {
        assert_eq!(act.pvm_ty(), &Actor);
        if let EditSession = ent.pvm_ty() {
            self.open_cache
                .get_mut(&ent.uuid())
                .unwrap()
                .remove(&act.uuid());
            if self.open_cache[&ent.uuid()].is_empty() {
                let f = self.add(
                    Store,
                    ent.ty(),
                    ent.uuid(),
                    Some(ent.meta.snapshot(&self.cur_time)),
                );
                self._inf(ent, &*f, PVMOps::Version);
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

    fn decl_fcont(&mut self, ent: &DataNode) -> NodeGuard {
        let id = {
            if !self.cont_cache.contains_key(&ent.uuid()) {
                let id = self._nextid();
                let node = DataNode::new(StoreCont, ent.ty(), id, ent.uuid(), None);
                self.cont_cache.insert(ent.uuid(), id);
                self.db.create_node(&node);
                self.node_cache.insert(id, node);
                id
            } else {
                self.cont_cache[&ent.uuid()]
            }
        };
        self.node_cache.lend(&id).unwrap()
    }

    pub fn name(&mut self, obj: &DataNode, name: Name) -> RelGuard {
        let n_node = self.decl_name(name);
        match obj.pvm_ty() {
            //          Store | EditSession => {
            //              let cont = self.decl_fcont(obj);
            //              self._named(&*cont, &n_node)
            //          }
            _ => self._named(obj, &n_node),
        }
    }

    pub fn unname(&mut self, obj: &DataNode, name: Name) -> RelGuard {
        let mut rel = self.name(obj, name);
        if let Rel::Named(ref mut n_rel) = *rel {
            n_rel.end = self.cur_time;
            self.db.update_rel(&*rel);
        }
        rel
    }

    pub fn meta<T: ToString + ?Sized>(
        &mut self,
        ent: &mut DataNode,
        key: &'static str,
        val: &T,
    ) -> Result<(), PVMError> {
        if !ent.ty().props.contains_key(key) {
            return Err(PVMError::CannotAssignMeta {
                ty: ent.ty(),
                key,
                value: val.to_string(),
            });
        }
        ent.meta
            .update(key, val, &self.cur_time, ent.ty().props[key]);
        self.db.update_node(ent);
        Ok(())
    }

    pub fn prop(&mut self, ent: &DataNode) {
        self.db.update_node(ent)
    }

    pub fn connect(&mut self, first: &DataNode, second: &DataNode, dir: ConnectDir) {
        assert_eq!(first.pvm_ty(), &Conduit);
        assert_eq!(second.pvm_ty(), &Conduit);
        self._inf(first, second, PVMOps::Connect);
        if let ConnectDir::BiDirectional = dir {
            self._inf(second, first, PVMOps::Connect);
        }
    }

    pub fn shutdown(self) {}
}
