use std::sync::mpsc::SyncSender;

use super::persist::DBTr;

use uuid::Uuid5;

pub struct DB {
    persist_pipe: SyncSender<DBTr>,
}

impl DB {
    pub fn create(pipe: SyncSender<DBTr>) -> DB{
        DB { persist_pipe: pipe }
    }

    pub fn create_node(
        &mut self,
        id: i64,
        uuid: Uuid5,
        pid: i32,
        cmdline: String,
    ) -> Result<(), &'static str> {
        self.persist_pipe
            .send(DBTr::CreateNode {
                id,
                uuid,
                pid,
                cmdline,
            })
            .map_err(|_| "Database worker closed queue unexpectadly")
    }

    pub fn create_rel(&mut self, src: i64, dst: i64, class: String) -> Result<(), &'static str> {
        self.persist_pipe
            .send(DBTr::CreateRel { src, dst, class })
            .map_err(|_| "Database worker closed queue unexpectadly")
    }

    pub fn update_node(&mut self, id: i64, pid: i32, cmdline: String) -> Result<(), &'static str> {
        self.persist_pipe
            .send(DBTr::UpdateNode { id, pid, cmdline })
            .map_err(|_| "Database worker closed queue unexpectadly")
    }
}
