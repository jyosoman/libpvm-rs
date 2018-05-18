use chrono::{serde::ts_nano_seconds, DateTime, Utc};
use std::fmt;
use uuid::Uuid;

fn format_uuid(v: &Uuid) -> String {
    v.hyphenated().to_string()
}

fn format_opt_uuid(v: &Option<Uuid>) -> Option<String> {
    v.map(|u| format_uuid(&u))
}

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

impl fmt::Display for AuditEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entry(&"event", &self.event)
            .entry(&"host", &format_opt_uuid(&self.host))
            .entry(&"time", &self.time.to_rfc3339())
            .entry(&"pid", &self.pid)
            .entry(&"ppid", &self.ppid)
            .entry(&"tid", &self.tid)
            .entry(&"uid", &self.uid)
            .entry(&"cpu_id", &self.cpu_id)
            .entry(&"exec", &self.exec)
            .entry(&"cmdline", &self.cmdline)
            .entry(&"upath1", &self.upath1)
            .entry(&"upath2", &self.upath2)
            .entry(&"fd", &self.fd)
            .entry(&"flags", &self.flags)
            .entry(&"fdpath", &self.fdpath)
            .entry(&"subjprocuuid", &format_uuid(&self.subjprocuuid))
            .entry(&"subjthruuid", &format_uuid(&self.subjthruuid))
            .entry(&"arg_objuuid1", &format_opt_uuid(&self.arg_objuuid1))
            .entry(&"arg_objuuid2", &format_opt_uuid(&self.arg_objuuid2))
            .entry(&"ret_objuuid1", &format_opt_uuid(&self.ret_objuuid1))
            .entry(&"ret_objuuid2", &format_opt_uuid(&self.ret_objuuid2))
            .entry(&"retval", &self.retval)
            .entry(&"ret_fd1", &self.ret_fd1)
            .entry(&"ret_fd2", &self.ret_fd2)
            .entry(&"arg_mem_flags", &self.arg_mem_flags)
            .entry(&"arg_sharing_flags", &self.arg_sharing_flags)
            .entry(&"address", &self.address)
            .entry(&"port", &self.port)
            .finish()
    }
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

impl fmt::Display for FBTEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entry(&"event", &self.event)
            .entry(&"host", &format_uuid(&self.host))
            .entry(&"time", &self.time.to_rfc3339())
            .entry(&"so_uuid", &format_uuid(&self.so_uuid))
            .entry(&"lport", &self.lport)
            .entry(&"fport", &self.fport)
            .entry(&"laddr", &self.laddr)
            .entry(&"faddr", &self.faddr)
            .finish()
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum TraceEvent {
    Audit(Box<AuditEvent>),
    FBT(FBTEvent),
}

impl fmt::Display for TraceEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TraceEvent::Audit(box ae) => {
                write!(f, "TraceEvent::Audit(")?;
                <AuditEvent as fmt::Display>::fmt(ae, f)?;
                write!(f, ")")
            }
            TraceEvent::FBT(fbt) => {
                write!(f, "TraceEvent::FBT(")?;
                <FBTEvent as fmt::Display>::fmt(fbt, f)?;
                write!(f, ")")
            }
        }
    }
}
