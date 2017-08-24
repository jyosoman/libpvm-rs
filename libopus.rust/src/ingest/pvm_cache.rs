use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use uuid::Uuid5;

pub struct PVMCache {
    node_cache: HashMap<Uuid5, Node>,
    id_counter: AtomicUsize,
}

pub struct Node {
    pub db_id: i64,
    pub cmdline: String,
    pub thin: bool,
}

impl PVMCache {
    pub fn new() -> PVMCache {
        PVMCache {
            node_cache: HashMap::new(),
            id_counter: AtomicUsize::new(0),
        }
    }

    pub fn add(&mut self, uuid: Uuid5, cmdline: &str, thin: bool) {
        let node = Node {
            db_id: self.id_counter.fetch_add(1, Ordering::SeqCst) as i64,
            cmdline: String::from(cmdline),
            thin: thin,
        };
        self.node_cache.insert(uuid, node);
    }

    pub fn check(&mut self, uuid: Uuid5, cmdline: &str) -> bool {
        if !self.node_cache.contains_key(&uuid) {
            self.add(uuid, cmdline, true);
            true
        } else {
            false
        }
    }

    pub fn get(&self, uuid: Uuid5) -> &Node {
        &self.node_cache[&uuid]
    }

    pub fn set(&mut self, uuid: Uuid5, cmdline: &str, thin: bool) {
        let node = self.node_cache.get_mut(&uuid).unwrap();
        node.cmdline = String::from(cmdline);
        node.thin = thin;
    }
}
