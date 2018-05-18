use chrono::{serde::ts_nano_seconds, DateTime, Utc};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct AuditEvent {
    pub event: String,
    pub host: Option<Uuid>,
    #[serde(with = "ts_nano_seconds")]
    pub time: DateTime<Utc>,
    pub pid: i32,
    pub ppid: i32,
    pub tid: i32,
    pub uid: i32,
    pub cpu_id: Option<i32>,
    pub exec: String,
    pub cmdline: Option<String>,
    pub upath1: Option<String>,
    pub upath2: Option<String>,
    pub fd: Option<i32>,
    pub flags: Option<i32>,
    pub fdpath: Option<String>,
    pub subjprocuuid: Uuid,
    pub subjthruuid: Uuid,
    pub arg_objuuid1: Option<Uuid>,
    pub arg_objuuid2: Option<Uuid>,
    pub ret_objuuid1: Option<Uuid>,
    pub ret_objuuid2: Option<Uuid>,
    pub retval: i32,
    pub ret_fd1: Option<i32>,
    pub ret_fd2: Option<i32>,
    pub arg_mem_flags: Option<Vec<String>>,
    pub arg_sharing_flags: Option<Vec<String>>,
    pub address: Option<String>,
    pub port: Option<u16>,
}

#[derive(Deserialize, Debug)]
pub struct FBTEvent {
    pub event: String,
    pub host: Uuid,
    #[serde(with = "ts_nano_seconds")]
    pub time: DateTime<Utc>,
    pub so_uuid: Uuid,
    pub lport: i32,
    pub fport: i32,
    pub laddr: String,
    pub faddr: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum TraceEvent {
    Audit(Box<AuditEvent>),
    FBT(FBTEvent),
}
