use data::{Generable, HasID, HasUUID, NodeID};
use uuid::Uuid5;

pub struct PipeInit {
    pub fd: i32,
}

#[derive(Clone, Debug)]
pub struct Pipe {
    db_id: NodeID,
    uuid: Uuid5,
    pub fd: i32,
}

impl HasID for Pipe {
    fn get_db_id(&self) -> NodeID {
        self.db_id
    }
}

impl Generable for Pipe {
    type Init = PipeInit;

    fn new(id: NodeID, uuid: Uuid5, init: Option<Self::Init>) -> Self
    where
        Self: Sized,
    {
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
    fn get_uuid(&self) -> Uuid5 {
        self.uuid
    }
}
