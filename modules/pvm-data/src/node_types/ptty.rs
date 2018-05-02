use ::{Generable, HasID, HasUUID, ID};
use uuid::Uuid;

pub struct PttyInit {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Ptty {
    db_id: ID,
    uuid: Uuid,
    pub name: String,
}

impl HasID for Ptty {
    fn get_db_id(&self) -> ID {
        self.db_id
    }
}

impl Generable for Ptty {
    type Init = PttyInit;

    fn new(id: ID, uuid: Uuid, init: Option<Self::Init>) -> Self
        where
            Self: Sized,
    {
        match init {
            Some(i) => Ptty {
                db_id: id,
                uuid,
                name: i.name,
            },
            None => Ptty {
                db_id: id,
                uuid,
                name: String::new(),
            },
        }
    }
}

impl HasUUID for Ptty {
    fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}
