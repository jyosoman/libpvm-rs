use uuid::Uuid5;

#[derive(Deserialize, Debug)]
pub struct AuditEvent {
    pub event: String,
    pub host: Option<Uuid5>,
    pub time: u64,
    pub pid: i32,
    pub ppid: i32,
    pub tid: i32,
    pub uid: i32,
    pub exec: Option<String>,
    pub cmdline: Option<String>,
    pub upath1: Option<String>,
    pub upath2: Option<String>,
    pub fd: Option<i32>,
    pub flags: Option<i32>,
    pub fdpath: Option<String>,
    pub subjprocuuid: Uuid5,
    pub subjthruuid: Uuid5,
    pub arg_objuuid1: Option<Uuid5>,
    pub arg_objuuid2: Option<Uuid5>,
    pub ret_objuuid1: Option<Uuid5>,
    pub ret_objuuid2: Option<Uuid5>,
    pub retval: i32,
    pub arg_mem_flags: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct FBTEvent {
    pub event: String,
    pub host: Uuid5,
    pub time: u64,
    pub so_uuid: Uuid5,
    pub lport: i32,
    pub fport: i32,
    pub laddr: String,
    pub faddr: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum TraceEvent {
    Audit(AuditEvent),
    FBT(FBTEvent),
}
