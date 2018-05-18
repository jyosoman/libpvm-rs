use uuid::Uuid;
use {Generable, HasID, HasUUID, ID};

#[derive(Clone, Debug)]
pub struct Ptty {
    db_id: ID,
    uuid: Uuid,
}

impl HasID for Ptty {
    fn get_db_id(&self) -> ID {
        self.db_id
    }
}

impl Generable for Ptty {
    type Init = !;

    fn new(id: ID, uuid: Uuid, _init: Option<Self::Init>) -> Self {
        Ptty { db_id: id, uuid }
    }
}

impl HasUUID for Ptty {
    fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}
