use std::sync::mpsc::SyncSender;

use super::persist::DBTr;
use super::Node;

pub struct DB {
    persist_pipe: SyncSender<DBTr>,
}

impl DB {
    pub fn create(pipe: SyncSender<DBTr>) -> DB {
        DB { persist_pipe: pipe }
    }

    pub fn create_node(&mut self, node: &Node) {
        self.persist_pipe
            .send(DBTr::CreateNode {
                id: node.db_id,
                uuid: node.uuid,
                pid: node.pid,
                cmdline: node.cmdline.clone(),
            })
            .expect("Database worker closed queue unexpectadly")
    }

    pub fn create_rel(&mut self, src: &Node, dst: &Node, class: String) {
        self.persist_pipe
            .send(DBTr::CreateRel {
                src: src.db_id,
                dst: dst.db_id,
                class,
            })
            .expect("Database worker closed queue unexpectadly")
    }

    pub fn update_node(&mut self, node: &Node) {
        self.persist_pipe
            .send(DBTr::UpdateNode {
                id: node.db_id,
                pid: node.pid,
                cmdline: node.cmdline.clone(),
            })
            .expect("Database worker closed queue unexpectadly")
    }
}
