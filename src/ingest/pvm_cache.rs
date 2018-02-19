use std::collections::HashMap;

use std::sync::atomic::{AtomicUsize, Ordering};

use uuid::Uuid5;

use super::Node;

use checking_store::{CheckingStore, DropGuard};

pub type NodeGuard = DropGuard<i64, Node>;

pub struct PVMCache {
    uuid_cache: HashMap<Uuid5, i64>,
    node_cache: CheckingStore<i64, Node>,
    id_counter: AtomicUsize,
}

impl PVMCache {
    pub fn new() -> PVMCache {
        PVMCache {
            uuid_cache: HashMap::new(),
            node_cache: CheckingStore::new(),
            id_counter: AtomicUsize::new(0),
        }
    }

    pub fn add(&mut self, uuid: Uuid5, pid: i32, cmdline: &str, thin: bool) -> NodeGuard {
        let id = self.id_counter.fetch_add(1, Ordering::SeqCst) as i64;
        let node = Node {
            db_id: id,
            uuid,
            pid,
            cmdline: String::from(cmdline),
            thin,
        };
        self.uuid_cache.insert(uuid, id);
        self.node_cache.insert(id, node);
        self.node_cache.checkout(&id).unwrap()
    }

    pub fn check(&mut self, uuid: Uuid5, pid: i32, cmdline: &str) -> (bool, NodeGuard) {
        if !self.uuid_cache.contains_key(&uuid) {
            (true, self.add(uuid, pid, cmdline, true))
        } else {
            (
                false,
                self.node_cache.checkout(&self.uuid_cache[&uuid]).unwrap(),
            )
        }
    }

    pub fn release(&mut self, uuid: &Uuid5) {
        self.uuid_cache.remove(uuid);
    }

    //pub fn checkout(&mut self, uuid: &Uuid5) ->  NodeGuard{
    //    self.node_cache.checkout(&self.uuid_cache[uuid]).unwrap()
    //}

    pub fn checkin(&mut self, guard: NodeGuard) {
        self.node_cache.checkin(guard)
    }
}
