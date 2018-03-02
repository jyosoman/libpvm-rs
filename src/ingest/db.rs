use std::sync::mpsc::SyncSender;

use super::persist::DBTr;
use data::{HasID, ToDB};

use packstream::values::Value;

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
        self.persist_pipe
            .send(DBTr::CreateNode(
                node.get_db_id(),
                node.get_labels(),
                node.to_db(),
            ))
            .expect("Database worker closed queue unexpectadly")
    }

    pub fn create_rel<T, U>(&mut self, src: &T, dst: &U, rtype: &'static str, class: &'static str)
    where
        T: HasID,
        U: HasID,
    {
        let props = hashmap!("src"   => Value::from(src.get_db_id()),
                             "dst"   => Value::from(dst.get_db_id()),
                             "type"  => Value::from(rtype),
                             "class" => Value::from(class));
        self.persist_pipe
            .send(DBTr::CreateRel(props.into()))
            .expect("Database worker closed queue unexpectadly");
    }

    pub fn update_node<T>(&mut self, node: &T)
    where
        T: ToDB + HasID,
    {
        self.persist_pipe
            .send(DBTr::UpdateNode(node.get_db_id(), node.to_db()))
            .expect("Database worker closed queue unexpectadly")
    }
}
