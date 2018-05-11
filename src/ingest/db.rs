use std::sync::mpsc::SyncSender;

use data::{Enumerable, Rel};
use views::DBTr;

pub struct DB {
    persist_pipe: SyncSender<DBTr>,
}

impl DB {
    pub fn create(pipe: SyncSender<DBTr>) -> DB {
        DB { persist_pipe: pipe }
    }

    pub fn create_node(&mut self, node: impl Enumerable) {
        self.persist_pipe
            .send(DBTr::CreateNode(node.enumerate()))
            .expect("Database worker closed queue unexpectadly")
    }

    pub fn create_rel(&mut self, rel: &Rel) {
        self.persist_pipe
            .send(DBTr::CreateRel(rel.clone()))
            .expect("Database worker closed queue unexpectadly");
    }

    pub fn update_node(&mut self, node: impl Enumerable) {
        self.persist_pipe
            .send(DBTr::UpdateNode(node.enumerate()))
            .expect("Database worker closed queue unexpectadly")
    }

    pub fn update_rel(&mut self, rel: &Rel) {
        self.persist_pipe
            .send(DBTr::UpdateRel(rel.clone()))
            .expect("Database worker closed queue unexpectadly")
    }
}
