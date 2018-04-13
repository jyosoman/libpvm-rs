use data::{Generable, HasID, HasUUID, ID};
use uuid::Uuid5;

pub struct FileInit {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct File {
    db_id: ID,
    uuid: Uuid5,
    pub name: String,
}

impl HasID for File {
    fn get_db_id(&self) -> ID {
        self.db_id
    }
}

impl Generable for File {
    type Init = FileInit;

    fn new(id: ID, uuid: Uuid5, init: Option<Self::Init>) -> Self
    where
        Self: Sized,
    {
        match init {
            Some(i) => File {
                db_id: id,
                uuid,
                name: i.name,
            },
            None => File {
                db_id: id,
                uuid,
                name: String::new(),
            },
        }
    }
}

impl HasUUID for File {
    fn get_uuid(&self) -> Uuid5 {
        self.uuid
    }
}
