use ::{Generable, HasID, HasUUID, ID};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub enum SocketClass {
    Unknown = 0,
    AfInet = 1,
    AfUnix = 2,
}

impl SocketClass {
    pub fn from_int(val: i64) -> Option<SocketClass> {
        match val {
            0 => Some(SocketClass::Unknown),
            1 => Some(SocketClass::AfInet),
            2 => Some(SocketClass::AfUnix),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Socket {
    db_id: ID,
    uuid: Uuid,
    pub class: SocketClass,
    pub path: String,
    pub ip: String,
    pub port: u16,
}

pub struct SocketInit {
    pub class: SocketClass,
    pub path: String,
    pub ip: String,
    pub port: u16,
}

impl HasID for Socket {
    fn get_db_id(&self) -> ID {
        self.db_id
    }
}

impl Generable for Socket {
    type Init = SocketInit;

    fn new(id: ID, uuid: Uuid, init: Option<Self::Init>) -> Self
    where
        Self: Sized,
    {
        match init {
            Some(i) => Socket {
                db_id: id,
                uuid,
                class: i.class,
                path: i.path,
                ip: i.ip,
                port: i.port,
            },
            None => Socket {
                db_id: id,
                uuid,
                class: SocketClass::Unknown,
                path: String::new(),
                ip: String::new(),
                port: 0,
            },
        }
    }
}

impl HasUUID for Socket {
    fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}
