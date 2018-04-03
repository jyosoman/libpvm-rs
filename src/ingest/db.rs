use std::sync::mpsc::SyncSender;

use super::persist::DBTr;
use data::{Enumerable, HasID};

use neo4j::Value;

pub struct DB {
    persist_pipe: SyncSender<DBTr>,
}

impl DB {
    pub fn create(pipe: SyncSender<DBTr>) -> DB {
        DB { persist_pipe: pipe }
    }

    pub fn create_node<T>(&mut self, node: T)
    where
        T: Enumerable,
    {
        self.persist_pipe
            .send(DBTr::CreateNode(node.enumerate()))
            .expect("Database worker closed queue unexpectadly")
    }

    pub fn create_rel<T, U>(&mut self, src: &T, dst: &U, rtype: &'static str, class: &str)
    where
        T: HasID,
        U: HasID,
    {
        self.persist_pipe
            .send(DBTr::CreateRel {
                src: src.get_db_id(),
                dst: dst.get_db_id(),
                ty: rtype,
                props: hashmap!("class" => Value::from(class)),
            })
            .expect("Database worker closed queue unexpectadly");
    }

    pub fn update_node<T>(&mut self, node: T)
    where
        T: Enumerable,
    {
        self.persist_pipe
            .send(DBTr::UpdateNode(node.enumerate()))
            .expect("Database worker closed queue unexpectadly")
    }
}
