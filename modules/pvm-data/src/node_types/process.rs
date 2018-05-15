use uuid::Uuid;
use {Generable, HasID, HasUUID, ID};

#[derive(Clone, Debug)]
pub struct Process {
    db_id: ID,
    uuid: Uuid,
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
    fn get_db_id(&self) -> ID {
        self.db_id
    }
}

impl Generable for Process {
    type Init = ProcessInit;

    fn new(id: ID, uuid: Uuid, init: Option<Self::Init>) -> Self
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
    fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}
