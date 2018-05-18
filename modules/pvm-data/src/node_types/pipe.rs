use uuid::Uuid;
use {Generable, HasID, HasUUID, ID};

pub struct PipeInit {
    pub fd: i32,
}

#[derive(Clone, Debug)]
pub struct Pipe {
    db_id: ID,
    uuid: Uuid,
    pub fd: i32,
}

impl HasID for Pipe {
    fn get_db_id(&self) -> ID {
        self.db_id
    }
}

impl Generable for Pipe {
    type Init = PipeInit;

    fn new(id: ID, uuid: Uuid, init: Option<Self::Init>) -> Self {
        match init {
            Some(i) => Pipe {
                db_id: id,
                uuid,
                fd: i.fd,
            },
            None => Pipe {
                db_id: id,
                uuid,
                fd: -1,
            },
        }
    }
}

impl HasUUID for Pipe {
    fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}
