use uuid::Uuid;
use {meta_store::MetaStore, Generable, HasID, HasUUID, ID};

#[derive(Clone, Debug)]
pub struct Process {
    db_id: ID,
    uuid: Uuid,
    pub meta: MetaStore,
}

impl HasID for Process {
    fn get_db_id(&self) -> ID {
        self.db_id
    }
}

impl Generable for Process {
    type Init = MetaStore;

    fn new(id: ID, uuid: Uuid, init: Option<Self::Init>) -> Self {
        match init {
            Some(i) => Process {
                db_id: id,
                uuid,
                meta: i,
            },
            None => Process {
                db_id: id,
                uuid,
                meta: MetaStore::new(),
            },
        }
    }
}

impl HasUUID for Process {
    fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}
