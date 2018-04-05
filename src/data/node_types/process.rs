use data::{Generable, HasID, HasUUID, NodeID};
use uuid::Uuid5;

#[derive(Clone, Debug)]
pub struct Process {
    db_id: NodeID,
    uuid: Uuid5,
    pub pid: i32,
    pub cmdline: String,
    pub thin: bool,
}

pub struct ProcessInit {
    pub pid: i32,
    pub cmdline: String,
    pub thin: bool,
}

impl HasID for Process {
    fn get_db_id(&self) -> NodeID {
        self.db_id
    }
}

impl Generable for Process {
    type Init = ProcessInit;

    fn new(id: NodeID, uuid: Uuid5, init: Option<Self::Init>) -> Self
    where
        Self: Sized,
    {
        match init {
            Some(i) => Process {
                db_id: id,
                uuid,
                cmdline: i.cmdline,
                pid: i.pid,
                thin: i.thin,
            },
            None => Process {
                db_id: id,
                uuid,
                cmdline: String::new(),
                pid: 0,
                thin: true,
            },
        }
    }
}

impl HasUUID for Process {
    fn get_uuid(&self) -> Uuid5 {
        self.uuid
    }
}
