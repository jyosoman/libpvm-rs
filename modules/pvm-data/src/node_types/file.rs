use uuid::Uuid;
use {Generable, HasID, HasUUID, ID};

#[derive(Clone, Debug)]
pub struct File {
    db_id: ID,
    uuid: Uuid,
}

impl HasID for File {
    fn get_db_id(&self) -> ID {
        self.db_id
    }
}

impl Generable for File {
    type Init = !;

    fn new(id: ID, uuid: Uuid, _init: Option<Self::Init>) -> Self
    where
        Self: Sized,
    {
        File { db_id: id, uuid }
    }
}

impl HasUUID for File {
    fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}
