use std::sync::mpsc::SyncSender;

use super::persist::DBTr;
use data::{HasID, ToDB};

use neo4j::Value;

pub struct DB {
    persist_pipe: SyncSender<DBTr>,
}

impl DB {
    pub fn create(pipe: SyncSender<DBTr>) -> DB {
        DB { persist_pipe: pipe }
    }

    pub fn create_node<T>(&mut self, node: &T)
    where
        T: ToDB + HasID,
    {
        let (id, labs, props) = node.to_db();
        self.persist_pipe
            .send(DBTr::CreateNode { id, labs, props })
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

    pub fn update_node<T>(&mut self, node: &T)
    where
        T: ToDB + HasID,
    {
        let (id, _, props) = node.to_db();
        self.persist_pipe
            .send(DBTr::UpdateNode { id, props })
            .expect("Database worker closed queue unexpectadly")
    }
}
