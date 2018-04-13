use data::{Generable, HasID, HasUUID, ID};
use uuid::Uuid5;

#[derive(Clone, Debug)]
pub struct EditSession {
    db_id: ID,
    uuid: Uuid5,
    pub name: String,
}

pub struct EditInit {
    pub name: String,
}

impl HasID for EditSession {
    fn get_db_id(&self) -> ID {
        self.db_id
    }
}

impl Generable for EditSession {
    type Init = EditInit;

    fn new(id: ID, uuid: Uuid5, init: Option<Self::Init>) -> Self
    where
        Self: Sized,
    {
        match init {
            Some(i) => EditSession {
                db_id: id,
                uuid,
                name: i.name,
            },
            None => EditSession {
                db_id: id,
                uuid,
                name: String::new(),
            },
        }
    }
}

impl HasUUID for EditSession {
    fn get_uuid(&self) -> Uuid5 {
        self.uuid
    }
}
