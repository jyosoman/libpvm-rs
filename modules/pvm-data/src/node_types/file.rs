use uuid::Uuid;
use {Generable, HasID, HasUUID, ID};

#[derive(Clone, Debug)]
pub struct File {
    db_id: ID,
    uuid: Uuid,
}

#[derive(Clone, Debug)]
pub struct FileContainer {
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

    fn new(id: ID, uuid: Uuid, _init: Option<Self::Init>) -> Self {
        File { db_id: id, uuid }
    }
}

impl HasUUID for File {
    fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}

impl FileContainer {
    pub fn new(id: ID, uuid: Uuid) -> Self {
        FileContainer { db_id: id, uuid }
    }
}

impl HasID for FileContainer {
    fn get_db_id(&self) -> ID {
        self.db_id
    }
}

impl HasUUID for FileContainer {
    fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}
