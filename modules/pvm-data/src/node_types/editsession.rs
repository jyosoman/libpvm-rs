use uuid::Uuid;
use {Generable, HasID, HasUUID, ID};

#[derive(Clone, Debug)]
pub struct EditSession {
    db_id: ID,
    uuid: Uuid,
}

impl HasID for EditSession {
    fn get_db_id(&self) -> ID {
        self.db_id
    }
}

impl Generable for EditSession {
    type Init = !;

    fn new(id: ID, uuid: Uuid, _init: Option<Self::Init>) -> Self {
        EditSession { db_id: id, uuid }
    }
}

impl HasUUID for EditSession {
    fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}
